use crate::compressor::Compressor;
use anyhow::Result;
use voxell_timer::time_fn;

if_tracing! {
    use tracing::{span, info};
}

pub mod arcode;
pub mod bwt;
pub mod dispatched;
pub mod huffman;
pub mod mtf;
pub mod pipeline;
pub mod re_pair;

/// All algorithms available in the current build of stackpack.
#[rustfmt::skip]
pub const ALL_COMPRESSORS: &[DynCompressor] = &[
    arcode::ThisCompressor,
    bwt::ThisCompressor,
    mtf::ThisCompressor,
    re_pair::ThisCompressor,
];

#[derive(Clone, Copy, Debug)]
pub struct DynCompressor {
    pub compress: fn(data: &[u8], buf: &mut Vec<u8>),
    pub decompress: fn(data: &[u8], buf: &mut Vec<u8>) -> Result<()>,
}

impl Compressor for DynCompressor {
    fn compress_bytes(&mut self, data: &[u8], buf: &mut Vec<u8>) {
        if_tracing! {
            let span = tracing::span!(tracing::Level::INFO, "compressor", kind = "dyn", func = "dyn_compress");
            let _enter = span.enter();
        }
        let ((), d) = time_fn(|| (self.compress)(data, buf));
        if_tracing! {
            info!(elapsed_ms = %d.as_micros(), out_len = buf.len(), "dyn compress finished");
        }
    }

    fn decompress_bytes(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let span = tracing::span!(tracing::Level::INFO, "compressor", kind = "dyn", func = "dyn_decompress");
            let _enter = span.enter();
        }
        let (r, d) = time_fn(|| (self.decompress)(data, buf));
        if_tracing! {
            info!(elapsed_ms = %d.as_micros(), out_len = buf.len(), "dyn decompress finished");
        }
        r
    }
}
