use crate::{
    algorithms::DynMutator,
    mutator::{Mutator, Result},
};
use core::mem;
use std::fmt::Debug;
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

pub fn get_specific_compressor_from_name(s: &str) -> Option<DynMutator> {
    for comp in crate::algorithms::ALL_COMPRESSORS.iter() {
        if comp.name == s {
            return Some(comp.mutator);
        }
    }

    None
}
