use crate::compressor::{Compressor, Result};

/// A compression pipeline consisting of multiple compression algorithms.
///
/// Supports seamlessly compressing and decompressing a given byte array through a series of algorithms.
/// This is done by chaining the algorithms together, where the output of one algorithm becomes the input for the next.
///
/// For decompression, the order of algorithms is reversed, ensuring that the data is returned to its original form.
/// This can be done thanks to the guarantees [`Compressor`][Compressor] provides, such that each algorithm can decode
/// it's own encoded data back to the original data.
pub struct CompressionPipeline {
    pipeline: Vec<Box<dyn Compressor>>,
}

impl CompressionPipeline {
    /// Creates a new empty compression pipeline. This pipeline will return the data as-is
    /// until you add algorithms to it.
    pub const fn new() -> Self {
        Self { pipeline: vec![] }
    }

    /// Adds a new algorithm to the pipeline.
    pub fn push_algorithm<A: Compressor>(&mut self, algorithm: A) {
        self.pipeline.push(algorithm.into_boxed());
    }

    /// Adds a new algorithm to the pipeline.
    /// Chain this method to add multiple algorithms in a shorter way.
    pub fn with_algorithm<A: Compressor>(mut self, algorithm: A) -> Self {
        self.pipeline.push(algorithm.into_boxed());
        self
    }
}

impl Compressor for CompressionPipeline {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        if self.pipeline.is_empty() {
            return data.to_vec();
        }

        let mut compressed = vec![];

        // we do it this way because:
        // we dont want to allocate data in the heap before any compression takes place,
        // we want to avoid copying data around as much as possible,
        // and we want to reuse the allocation as much as possible.
        let mut reference = data;

        for algorithm in self.pipeline.iter_mut() {
            let encoded = algorithm.compress_bytes(reference);
            compressed.clear();
            compressed.extend_from_slice(encoded.as_slice());
            reference = compressed.as_slice();
        }

        compressed
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if self.pipeline.is_empty() {
            return Ok(data.to_vec());
        }

        let mut decompressed = vec![];

        // we do it this way because:
        // we dont want to allocate data in the heap before any compression takes place,
        // we want to avoid copying data around as much as possible,
        // and we want to reuse the allocation as much as possible.
        let mut reference = data;

        for algorithm in self.pipeline.iter_mut().rev() {
            match algorithm.decompress_bytes(reference) {
                Ok(decoded) => {
                    decompressed.clear();
                    decompressed.extend_from_slice(decoded.as_slice());
                    reference = decompressed.as_slice();
                }

                Err(err) => return Err(err),
            }
        }

        Ok(decompressed)
    }

    fn compressor_name(&self) -> String {
        format!(
            "Compression Pipeline: {}",
            self.pipeline
                .iter()
                .enumerate()
                .map(|(i, algorithm)| format!("#{}: {}", i, algorithm.compressor_name()))
                .collect::<Vec<String>>()
                .join(" -> ")
        )
    }

    fn into_boxed(self) -> Box<dyn Compressor> {
        Box::new(self)
    }
}

#[cfg(test)]
mod tests {

    use crate::algorithms::{arith::ArithmeticCoding, bwt::Bwt, mtf::Mtf, rle::Rle};

    use super::*;

    #[test]
    fn roundtrip_tests() {
        let mut pipelines = vec![
            CompressionPipeline::new(),
            CompressionPipeline::new().with_algorithm(ArithmeticCoding),
            CompressionPipeline::new().with_algorithm(Rle { debug: true }),
            CompressionPipeline::new().with_algorithm(Bwt),
            CompressionPipeline::new().with_algorithm(Mtf),
            CompressionPipeline::new().with_algorithm(Rle { debug: true }).with_algorithm(Bwt),
            CompressionPipeline::new().with_algorithm(Bwt).with_algorithm(Mtf),
            CompressionPipeline::new().with_algorithm(Mtf).with_algorithm(Rle { debug: true }),
        ];

        for mut pipeline in pipelines {
            eprintln!("Testing pipeline {}", pipeline.compressor_name());
            crate::tests::roundtrip_test(pipeline);
        }
    }
}
