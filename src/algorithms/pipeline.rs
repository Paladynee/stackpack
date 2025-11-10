use crate::{
    algorithms::{DynMutator, arcode::ArithmeticCoding, bsc::Bsc, bwt::Bwt, mtf::Mtf},
    mutator::{Mutator, Result},
    registered::{ALL_COMPRESSORS, RegisteredCompressor},
};
use core::mem;
use core::{fmt::Debug, str};
use voxell_timer::time_fn;

if_tracing! {
    use tracing::{Level, span};
}

#[derive(Debug)]
pub struct CompressionPipeline {
    pipeline: Vec<DynMutator>,
}

impl CompressionPipeline {
    pub const fn new() -> Self {
        Self { pipeline: vec![] }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        const END_OF_PIPELINE: u8 = b'\0';
        const END_OF_ALGORITHM_NAME: u8 = b',';
        let mut pipeline = CompressionPipeline::new();
        let mut start = 0;
        let mut index = 0;
        while index < bytes.len() {
            let c = bytes[index];
            match c {
                END_OF_ALGORITHM_NAME => {
                    let name = str::from_utf8(&bytes[start..index]).ok()?;
                    let algo = get_specific_compressor_from_name(name)?;
                    pipeline.push_algorithm(algo.mutator);
                    start = index + 1;
                }
                END_OF_PIPELINE => {
                    let name = str::from_utf8(&bytes[start..index]).ok()?;
                    let algo = get_specific_compressor_from_name(name)?;
                    pipeline.push_algorithm(algo.mutator);
                    return Some(pipeline);
                }
                _ => {}
            }
            index += 1;
        }

        None
    }

    pub fn push_algorithm(&mut self, algorithm: DynMutator) {
        self.pipeline.push(algorithm);
    }

    /// Chain this method to add multiple algorithms in a shorter way.
    pub fn with_algorithm(mut self, algorithm: DynMutator) -> Self {
        self.pipeline.push(algorithm);
        self
    }
}

impl Mutator for CompressionPipeline {
    fn drive_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let pipeline_span = span!(Level::INFO, "compression_pipeline", stages = self.pipeline.len());
            let _enter = pipeline_span.enter();
        }
        match self.pipeline.len() {
            0 => Ok(()),
            1 => self.pipeline[0].drive_mutation(data, buf),
            n => {
                let mut intermediate: Vec<u8> = vec![];
                // first algorithm compresses from data to buf
                let (res, d) = time_fn(|| self.pipeline[0].drive_mutation(data, buf));
                res?;
                if_tracing! {
                    tracing::info!(stage = 0, elapsed_ms = %d.as_micros(), out_len = buf.len(), "stage complete");
                }

                'run_algos: {
                    let mut ref1 = &mut *buf;
                    let mut ref2 = &mut intermediate;

                    for algo in self.pipeline.iter_mut().skip(1) {
                        let (res, d) = time_fn(|| algo.drive_mutation(ref1, ref2));
                        res?;
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
                };

                Ok(())
            }
        }
    }

    fn revert_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let pipeline_span = span!(Level::INFO, "decompression_pipeline", stages = self.pipeline.len());
            let _enter = pipeline_span.enter();
        }

        match self.pipeline.len() {
            0 => Ok(()),
            1 => self.pipeline[0].revert_mutation(data, buf),
            n => {
                let mut intermediate: Vec<u8> = vec![];

                // first algorithm decompresses from data to buf
                let (res, dur) = time_fn(|| self.pipeline[n - 1].revert_mutation(data, buf));
                res?;
                if_tracing! {
                    tracing::info!(stage = n - 1, elapsed_ms = %dur.as_micros(), out_len = buf.len(), "stage complete");
                }

                'run_algos: {
                    let mut ref1 = &mut *buf;
                    let mut ref2 = &mut intermediate;

                    for algo in self.pipeline.iter_mut().rev().skip(1) {
                        let (res, dur) = time_fn(|| algo.revert_mutation(ref1, ref2));
                        res?;
                        if_tracing! {
                            tracing::info!(elapsed_ms = %dur.as_micros(), out_len = ref2.len(), "stage complete");
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

pub fn get_specific_compressor_from_name(s: &str) -> Option<&RegisteredCompressor> {
    ALL_COMPRESSORS.iter().find(|&comp| comp.name == s)
}

pub fn default_pipeline() -> CompressionPipeline {
    if_tracing! {
        tracing::info!(event = "using_default_pipeline", "using default compression pipeline");
    };
    CompressionPipeline::new()
        .with_algorithm(Bwt)
        .with_algorithm(Mtf)
        .with_algorithm(ArithmeticCoding)
}

pub fn bsc() -> CompressionPipeline {
    CompressionPipeline::new().with_algorithm(Bsc)
}

pub fn get_preset(s: &str) -> Option<fn() -> CompressionPipeline> {
    Some(match s {
        "default" => default_pipeline,
        "bsc" => bsc,
        _ => None?,
    })
}
