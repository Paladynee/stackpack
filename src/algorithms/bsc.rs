use core::ffi::c_int;

use crate::{algorithms::DynMutator, registered::RegisteredCompressor};
use anyhow::{Result, anyhow};
use bsc_m03_sys::{libbsc_compress_memory_block_u8, libbsc_decompress_memory_block_c};
use core::mem::size_of;

if_tracing! {
    use tracing::{debug, error, info, warn};
}

macro_rules! cold {
    ($(captures: $($Cap:ident: $CapTy:ty)*,)? $($Body:block)* -> $Ty:ty) => {{
        #[cold]
        fn __cold($($($Cap: $Capty)*)?) -> $Ty {
            $($Body)*
        }
        __cold($($($Cap)*)?)
    }};
}

pub const Bsc: RegisteredCompressor = RegisteredCompressor::new_dyn(
    DynMutator {
        drive_mutation: bsc_encode,
        revert_mutation: bsc_decode,
    },
    "bsc",
    Some(DESCRIPTION),
);
const DESCRIPTION: &str = "bsc-m03 general purpose compressor by Ilya Grebnov.";

fn bsc_encode(mut data: &[u8], output: &mut Vec<u8>) -> Result<()> {
    if_tracing! {
        tracing::debug!(target = "bsc", data.len = data.len(), "enter bsc encode");
    };
    output.clear();
    let mut remaining_size: i64 = data.len() as i64;
    let mut buffer_size = remaining_size.min(i32::MAX as i64) + 16384;
    buffer_size += buffer_size / 16;
    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size as usize);
    while remaining_size > 0 {
        // fits in i32 guaranteed, as max_block_size is i32 and we're doing a min
        let block_size: i32 = remaining_size.min(i32::MAX as i64) as i32;
        buffer.clear();
        let (block, rest) = data
            .split_at_checked(block_size as usize)
            .ok_or_else(|| cold!({ anyhow!("input too short") } -> anyhow::Error))?;
        buffer.extend_from_slice(block);
        data = rest;
        let compressed_size: i32 = unsafe { libbsc_compress_memory_block_u8(buffer.as_mut_ptr(), block_size as c_int) as i32 };
        if compressed_size <= 0 || compressed_size > block_size {
            return cold!({Err(anyhow!(
                "compression failed: internal error, please contact Ilya Grebnov, the author of bsc-m03 and libsais."
            ))} -> Result<()>);
        }
        unsafe {
            buffer.set_len(compressed_size as usize);
        };
        output.reserve((size_of::<i32>() as i32 + size_of::<i32>() as i32 + compressed_size) as usize);
        output.extend_from_slice(&block_size.to_le_bytes());
        output.extend_from_slice(&compressed_size.to_le_bytes());
        let s = &buffer[..compressed_size as usize];
        output.extend_from_slice(s);
        remaining_size -= block_size as i64;
    }
    if remaining_size != 0 {
        return cold!({Err(anyhow!(
            "internal error: remaining size after processing is not zero"
        ))} -> Result<()>);
    }
    Ok(())
}

fn bsc_decode(mut data: &[u8], output: &mut Vec<u8>) -> Result<()> {
    #[inline]
    fn read_i32(data: &mut &[u8]) -> Result<i32> {
        let (block, rest) = (*data)
            .split_at_checked(4)
            .ok_or_else(|| cold!({ anyhow!("input too short") } -> anyhow::Error))?;
        *data = rest;
        Ok(i32::from_le_bytes(block.try_into().unwrap()))
    }

    output.clear();
    if data.is_empty() {
        return Ok(());
    }

    let mut buffer: Vec<u8> = Vec::new();
    let mut remaining_size: i64 = data.len() as i64;

    while !data.is_empty() {
        let block_size: i32 = read_i32(&mut data)?;
        let compressed_size: i32 = read_i32(&mut data)?;
        if block_size <= 0 || compressed_size <= 0 || compressed_size > block_size {
            return cold!({ Err(anyhow!("corrupted input")) } -> Result<()>);
        }
        let block_size_usize = block_size as usize;
        let compressed_size_usize = compressed_size as usize;
        remaining_size -= (2 * size_of::<i32>()) as i64;
        let (compressed_slice, rest) = data
            .split_at_checked(compressed_size_usize)
            .ok_or_else(|| cold!({ anyhow!("input too short") } -> anyhow::Error))?;
        if buffer.capacity() < block_size_usize {
            buffer.reserve(block_size_usize.saturating_sub(buffer.len()));
        }
        buffer.clear();
        buffer.extend_from_slice(compressed_slice);
        data = rest;
        remaining_size -= compressed_size_usize as i64;
        let decompressed_size: i32 = unsafe {
            if compressed_size < block_size {
                libbsc_decompress_memory_block_c(buffer.as_mut_ptr(), compressed_size as c_int, block_size as c_int) as i32
            } else {
                block_size
            }
        };
        if decompressed_size != block_size {
            return cold!({ Err(anyhow!("corrupted input")) } -> Result<()>);
        }
        unsafe {
            buffer.set_len(decompressed_size as usize);
        };
        let s = &buffer[..block_size_usize];
        output.extend_from_slice(s);
    }

    if remaining_size != 0 {
        return cold!({ Err(anyhow!(
            "internal error: remaining size after processing is not zero"
        )) } -> Result<()>);
    }

    Ok(())
}
