use crate::algorithms::DynMutator;
use anyhow::{Result, anyhow};
use libsais::{BwtConstruction, ThreadCount, bwt::Bwt as LibsaisBwt, suffix_array::ExtraSpace, typestate::OwnedBuffer};

if_tracing! {
    use tracing::{debug, info};
}

pub const Bwt: DynMutator = DynMutator {
    drive_mutation: bwt_encode,
    revert_mutation: bwt_decode,
};

pub use self::Bwt as ThisMutator;

fn bwt_encode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    let use_fixed_threads = data.len() > 1_000_000;
    if_tracing! {
        debug!(target = "bwt", input_len = data.len(), use_fixed_threads, "bwt encode selecting thread strategy");
    }
    let res = BwtConstruction::for_text(data)
        .with_owned_temporary_array_buffer_and_extra_space32(ExtraSpace::Recommended)
        .multi_threaded(if use_fixed_threads {
            ThreadCount::fixed(12)
        } else {
            ThreadCount::openmp_default()
        })
        .run()
        .unwrap();

    buf.clear();
    let primary_index = res.primary_index();
    let primary_index = u32::try_from(primary_index).expect("primary index must fit into u32");
    let bwt_slice = res.bwt();
    if_tracing! {
        debug!(target = "bwt", primary_index, bwt_len = bwt_slice.len(), "bwt encode libsais complete");
    }
    buf.extend_from_slice(&primary_index.to_le_bytes());
    buf.extend_from_slice(bwt_slice);

    Ok(())
}

fn bwt_decode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    if_tracing! {
        debug!(target = "bwt", input_len = data.len(), "bwt decode start");
    }

    if data.len() < 4 {
        buf.clear();
        buf.extend_from_slice(data);
        return Ok(());
    }

    let mut index_bytes = [0u8; 4];
    index_bytes.copy_from_slice(&data[..4]);
    let primary_index = u32::from_le_bytes(index_bytes) as usize;
    let bwt_payload = &data[4..];

    if bwt_payload.is_empty() {
        buf.clear();
        return Ok(());
    }

    if primary_index >= bwt_payload.len() {
        return Err(anyhow!("Invalid primary index: {} (bwt length: {})", primary_index, bwt_payload.len()));
    }

    if_tracing! {
        debug!(target = "bwt", primary_index, payload_len = bwt_payload.len(), "bwt decode parsed header");
    }

    buf.clear();
    buf.resize(bwt_payload.len(), 0);
    let bwt_owned = bwt_payload.to_vec();

    // SAFETY: the primary index has been validated against the BWT payload, so they hopefully
    // follow the libsais BWT conventions or this is UB.
    let builder = unsafe { LibsaisBwt::<u8, OwnedBuffer>::from_parts(bwt_owned, primary_index) }
        .unbwt()
        .in_borrowed_text_buffer(buf.as_mut_slice())
        .with_owned_temporary_array_buffer32();

    let use_fixed_threads = bwt_payload.len() > 1_000_000;
    if_tracing! {
        debug!(target = "bwt", payload_len = bwt_payload.len(), use_fixed_threads, "bwt decode selecting thread strategy");
    }
    let result = if use_fixed_threads {
        builder.multi_threaded(ThreadCount::fixed(12)).run()
    } else {
        builder.multi_threaded(ThreadCount::openmp_default()).run()
    };

    result.map_err(|err| anyhow!("libsais unbwt failed: {:?}", err))?;

    if_tracing! {
        info!(target = "bwt", output_len = buf.len(), "bwt decode complete");
    }

    Ok(())
}

// /// Build a circular suffix array using the doubling algorithm.
// /// This avoids the pathological behavior of comparing full rotations
// /// for each comparison and is much faster on repetitive inputs.
// fn build_circular_suffix_array(data: &[u8]) -> Vec<usize> {
//     let n = data.len();
//     let mut sa: Vec<usize> = (0..n).collect();
//     // initial ranks = byte values
//     let mut rank: Vec<usize> = data.iter().map(|&b| b as usize).collect();
//     let mut tmp_rank = vec![0usize; n];
//     let mut k = 1usize;

//     while k < n {
//         // sort sa by (rank[i], rank[i+k]) pair
//         sa.sort_by(|&i, &j| {
//             let ri = rank[i];
//             let rj = rank[j];
//             if ri != rj {
//                 ri.cmp(&rj)
//             } else {
//                 let ri2 = rank[(i + k) % n];
//                 let rj2 = rank[(j + k) % n];
//                 ri2.cmp(&rj2)
//             }
//         });

//         tmp_rank[sa[0]] = 0;
//         for idx in 1..n {
//             let prev = sa[idx - 1];
//             let cur = sa[idx];
//             let prev_pair = (rank[prev], rank[(prev + k) % n]);
//             let cur_pair = (rank[cur], rank[(cur + k) % n]);
//             tmp_rank[cur] = tmp_rank[prev] + if cur_pair == prev_pair { 0 } else { 1 };
//         }

//         rank.copy_from_slice(&tmp_rank);

//         // early exit if all ranks are unique
//         if rank[sa[n - 1]] == n - 1 {
//             break;
//         }

//         k <<= 1;
//     }

//     sa
// }

// fn bwt_encode(data: &[u8], buf: &mut Vec<u8>) {
//     if_tracing! {
//         debug!(target = "bwt", input_len = data.len(), "bwt encode start");
//     }

//     // If the input is too short, just return the original data.
//     if data.len() < 4 {
//         if_tracing! {
//             debug!(target = "bwt", input_len = data.len(), "bwt encode passthrough: input too short");
//         }
//         buf.clear();
//         buf.extend_from_slice(data);
//         return;
//     }

//     // If input is too large, return original data
//     if data.len() > u32::MAX as usize {
//         if_tracing! {
//             warn!(target = "bwt", input_len = data.len(), "bwt encode passthrough: input too large");
//         }
//         buf.clear();
//         buf.extend_from_slice(data);
//         return;
//     }

//     let n = data.len();

//     // Fast path: if all bytes equal, the BWT is trivial.
//     if data.iter().all(|&b| b == data[0]) {
//         let mut output = Vec::with_capacity(4 + n);
//         output.extend_from_slice(&0u32.to_le_bytes());
//         // last column is same as input
//         output.extend_from_slice(data);
//         buf.clear();
//         buf.extend_from_slice(&output);
//         if_tracing! {
//             info!(target = "bwt", input_len = n, output_len = buf.len(), orig_index = 0, "bwt encode fast-path: all bytes equal");
//         }
//         return;
//     }

//     // Build circular suffix array using doubling algorithm (O(n log n) behaviour with small constants).
//     let sa = build_circular_suffix_array(data);

//     // Find the index of the original string (rotation starting at 0)
//     let orig_index = sa.iter().position(|&i| i == 0).expect("Rotation starting at 0 must exist");

//     // Build the transformed output with enough capacity for both index bytes and transformed data
//     let mut output = Vec::with_capacity(4 + n);
//     output.extend_from_slice(&(orig_index as u32).to_le_bytes());
//     for &rot in &sa {
//         let last_char = data[(rot + n - 1) % n];
//         output.push(last_char);
//     }

//     buf.clear();
//     buf.extend_from_slice(&output);

//     if_tracing! {
//         info!(target = "bwt", input_len = n, output_len = output.len(), orig_index, "bwt encode complete");
//     }
// }

// fn bwt_decode(data_slice: &[u8], buf: &mut Vec<u8>) -> Result<()> {
//     if_tracing! {
//         debug!(target = "bwt", input_len = data_slice.len(), "bwt decode start");
//     }

//     // return the original data if the input is shorter than 4 bytes
//     if data_slice.len() < 4 {
//         if_tracing! {
//             debug!(target = "bwt", input_len = data_slice.len(), "bwt decode passthrough: input too short");
//         }
//         buf.clear();
//         buf.extend_from_slice(data_slice);
//         return Ok(());
//     }

//     // Read the original index (first 4 bytes, little-endian).
//     let orig_index = u32::from_le_bytes([data_slice[0], data_slice[1], data_slice[2], data_slice[3]]) as usize;
//     let bwt_transformed = &data_slice[4..];
//     let n = bwt_transformed.len();

//     // If there's no actual data after the index, return empty
//     if n == 0 {
//         if_tracing! {
//             debug!(target = "bwt", "bwt decode passthrough: no payload after index");
//         }
//         buf.clear();
//         return Ok(());
//     }

//     // Validate that orig_index is within bounds
//     if orig_index >= n {
//         if_tracing! {
//             warn!(target = "bwt", orig_index, data_len = n, "bwt decode error: invalid original index");
//         }
//         return Err(anyhow!("Invalid original index: {} (data length: {})", orig_index, n));
//     }

//     // Build the frequency table for each byte.
//     let mut freq = [0usize; 256];
//     for &byte in bwt_transformed {
//         freq[byte as usize] += 1;
//     }

//     // Compute the starting position for each byte in the sorted first column.
//     let mut starts = [0usize; 256];
//     let mut sum = 0;
//     for b in 0..256 {
//         starts[b] = sum;
//         sum += freq[b];
//     }

//     // Build the LF-mapping: lf[i] gives the next row in the reconstruction.
//     let mut lf = vec![0usize; n];
//     let mut seen = [0usize; 256];
//     for (i, &byte) in bwt_transformed.iter().enumerate() {
//         lf[i] = starts[byte as usize] + seen[byte as usize];
//         seen[byte as usize] += 1;
//     }

//     // Reconstruct the original string using the LF mapping.
//     let mut result = vec![0u8; n];
//     let mut row = orig_index;
//     // Reconstruct in reverse order.
//     for byte in result.iter_mut().rev() {
//         *byte = bwt_transformed[row];
//         row = lf[row];
//     }

//     buf.clear();
//     buf.extend_from_slice(&result);
//     if_tracing! {
//         info!(target = "bwt", output_len = result.len(), orig_index, "bwt decode complete");
//     }
//     Ok(())
// }
