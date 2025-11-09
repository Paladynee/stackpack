use crate::{
    algorithms::DynCompressor,
    compressor::{Compressor, Result},
};
use core::mem;
use std::{fmt::Debug, time::Duration};
use voxell_timer::time_fn;

if_tracing! {
    use tracing::{Level, span};
}

#[derive(Debug)]
pub struct CompressionPipeline {
    pipeline: Vec<DynCompressor>,
}

impl CompressionPipeline {
    pub const fn new() -> Self {
        Self { pipeline: vec![] }
    }

    pub fn push_algorithm(&mut self, algorithm: DynCompressor) {
        self.pipeline.push(algorithm);
    }

    /// Chain this method to add multiple algorithms in a shorter way.
    pub fn with_algorithm(mut self, algorithm: DynCompressor) -> Self {
        self.pipeline.push(algorithm);
        self
    }
}

impl Compressor for CompressionPipeline {
    fn compress_bytes(&mut self, data: &[u8], buf: &mut Vec<u8>) {
        if_tracing! {
            let pipeline_span = span!(Level::INFO, "compression_pipeline", stages = self.pipeline.len());
            let _enter = pipeline_span.enter();
        }

        match self.pipeline.len() {
            0 => {}
            1 => self.pipeline[0].compress_bytes(data, buf),
            n => {
                let mut intermediate: Vec<u8> = vec![];
                // first algorithm compresses from data to buf
                let ((), d) = time_fn(|| self.pipeline[0].compress_bytes(data, buf));
                let mut stage_times: Vec<Duration> = Vec::with_capacity(n);
                stage_times.push(d);
                if_tracing! {
                    tracing::info!(stage = 0, elapsed_ms = %d.as_micros(), out_len = buf.len(), "stage complete");
                }

                'run_algos: {
                    let mut ref1 = &mut *buf;
                    let mut ref2 = &mut intermediate;

                    for algo in self.pipeline.iter_mut().skip(1) {
                        let ((), d) = time_fn(|| algo.compress_bytes(ref1, ref2));
                        stage_times.push(d);
                        if_tracing! {
                            tracing::info!(elapsed_ms = %d.as_micros(), out_len = ref2.len(), "stage complete");
                        }

                        // swap the references around (this is so cool)
                        mem::swap(&mut ref1, &mut ref2);
                    }
                    // If the env var STACKPACK_STAGE_TIMINGS is set, print per-stage durations.
                    if std::env::var("STACKPACK_STAGE_TIMINGS").is_ok() {
                        eprintln!("Compression pipeline stage timings (ms):");
                        for (i, t) in stage_times.iter().enumerate() {
                            eprintln!("  stage {}: {:.3}", i, t.as_secs_f64() * 1000.0);
                        }
                    }
                }

                // write intermediate into buf if it was not the last buffer to get written
                if n % 2 == 0 {
                    mem::swap(buf, &mut intermediate);
                }
            }
        }
    }

    fn decompress_bytes(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let pipeline_span = span!(Level::INFO, "decompression_pipeline", stages = self.pipeline.len());
            let _enter = pipeline_span.enter();
        }

        match self.pipeline.len() {
            0 => Ok(()),
            1 => self.pipeline[0].decompress_bytes(data, buf),
            n => {
                let mut intermediate: Vec<u8> = vec![];

                // first algorithm decompresses from data to buf
                let (r0, d0) = time_fn(|| self.pipeline[n - 1].decompress_bytes(data, buf));
                r0?;
                if_tracing! {
                    tracing::info!(stage = n - 1, elapsed_ms = %d0.as_micros(), out_len = buf.len(), "stage complete");
                }

                'run_algos: {
                    let mut ref1 = &mut *buf;
                    let mut ref2 = &mut intermediate;

                    for algo in self.pipeline.iter_mut().rev().skip(1) {
                        let (r, d) = time_fn(|| algo.decompress_bytes(ref1, ref2));
                        r?;
                        if_tracing! {
                            tracing::info!(elapsed_ms = %d.as_micros(), out_len = ref2.len(), "stage complete");
                        }

                        // swap the references around (this is so cool)
                        mem::swap(&mut ref1, &mut ref2);
                    }
                }

                // write intermediate into buf if it was not the last buffer to get written
                if n % 2 == 0 {
                    mem::swap(buf, &mut intermediate);
                }

                Ok(())
            }
        }
    }
}
