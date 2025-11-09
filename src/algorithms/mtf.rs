use crate::{algorithms::DynMutator, mutator::Result};

if_tracing! {
    use tracing::{debug, info};
}

pub const Mtf: DynMutator = DynMutator {
    drive_mutation: mtf_encode,
    revert_mutation: mtf_decode,
};

pub use self::Mtf as ThisMutator;

macro_rules! iota {
    ($ty:ty; $size:expr) => {
        const {
            let mut buf = [0; $size];
            let mut i = 0usize;
            while i < buf.len() {
                buf[i] = i as $ty;
                i += 1;
            }
            buf
        }
    };
}

pub fn mtf_encode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    if_tracing! {
        debug!(target = "mtf", input_len = data.len(), "mtf encode start");
    }
    if data.is_empty() {
        if_tracing! {
            debug!(target = "mtf", "mtf encode passthrough: input empty");
        }
        return Ok(());
    }

    buf.clear();
    buf.reserve(data.len());

    // maps index to byte value
    let mut alphabet: [u8; 256] = iota![u8; 256];
    // maps byte value to index to alphabet
    let mut pos: [u8; 256] = iota![u8; 256];
    for b in data.iter().copied() {
        let idx = pos[b as usize];
        buf.push(idx);

        // If it's already at front nothing to do.
        if idx == 0 {
            continue;
        };

        let byte = alphabet[idx as usize];
        alphabet.copy_within(0..idx as usize, 1);
        alphabet[0] = byte;

        for i in 1..=idx {
            let v = alphabet[i as usize];
            pos[v as usize] = i;
        }
        pos[byte as usize] = 0;
    }

    if_tracing! {
        info!(target = "mtf", input_len = data.len(), output_len = buf.len(), "mtf encode complete");
    }

    Ok(())
}

pub fn mtf_decode(encoded: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    if_tracing! {
        debug!(target = "mtf", input_len = encoded.len(), "mtf decode start");
    }
    // If input empty, nothing to do.
    if encoded.is_empty() {
        buf.clear();
        if_tracing! {
            debug!(target = "mtf", "mtf decode passthrough: input empty");
        }
        return Ok(());
    }

    buf.clear();
    buf.reserve(encoded.len());

    // maps from index to byte value
    let mut alphabet: [u8; 256] = iota![u8; 256];

    for idx in encoded.iter().copied() {
        let symbol = alphabet[idx as usize];
        buf.push(symbol);

        if idx == 0 {
            continue;
        }
        alphabet.copy_within(0..idx as usize, 1);
        alphabet[0] = symbol;
    }

    if_tracing! {
        info!(target = "mtf", input_len = encoded.len(), output_len = buf.len(), "mtf decode complete");
    }

    Ok(())
}
