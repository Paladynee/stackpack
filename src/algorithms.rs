use crate::mutator::Mutator;
use anyhow::Result;
use voxell_timer::time_fn;

if_tracing! {
    use tracing::{info};
}

pub mod arcode;
pub mod bwt;
pub mod huffman;
pub mod mtf;
pub mod pipeline;
pub mod re_pair;
pub mod serializing_algorithm;

/// All algorithms available in the current build of stackpack.
#[rustfmt::skip]
pub static ALL_COMPRESSORS: &[RegisteredCompressor] = &[
    RegisteredCompressor::new(arcode::ThisMutator, "arcode"),
    RegisteredCompressor::new(bwt::ThisMutator, "bwt"),
    RegisteredCompressor::new(mtf::ThisMutator, "mtf"),
    RegisteredCompressor::new(re_pair::ThisMutator, "re_pair"),
];

#[derive(Clone, Copy, Debug)]
pub struct DynMutator {
    pub drive_mutation: fn(data: &[u8], buf: &mut Vec<u8>) -> Result<()>,
    pub revert_mutation: fn(data: &[u8], buf: &mut Vec<u8>) -> Result<()>,
}

impl Mutator for DynMutator {
    fn drive_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let span = tracing::span!(tracing::Level::INFO, "compressor", kind = "dyn", func = "dyn_compress");
            let _enter = span.enter();
        }
        let (res, d) = time_fn(|| (self.drive_mutation)(data, buf));
        if_tracing! {
            info!(elapsed_ms = %d.as_micros(), out_len = buf.len(), "dyn compress finished");
        }
        res
    }

    fn revert_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let span = tracing::span!(tracing::Level::INFO, "compressor", kind = "dyn", func = "dyn_decompress");
            let _enter = span.enter();
        }
        let (r, d) = time_fn(|| (self.revert_mutation)(data, buf));
        if_tracing! {
            info!(elapsed_ms = %d.as_micros(), out_len = buf.len(), "dyn decompress finished");
        }
        r
    }
}

pub struct RegisteredCompressor {
    mutator: DynMutator,
    name: &'static str,
}

impl RegisteredCompressor {
    const fn new(mutator: DynMutator, name: &'static str) -> Self {
        Self { mutator, name }
    }
}
