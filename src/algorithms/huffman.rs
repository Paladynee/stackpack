use anyhow::anyhow;

use crate::compressor::{Compressor, CompressorExt, DecompressionError, Result};

#[derive(Clone)]
pub struct HuffmanCoding;

impl Compressor for HuffmanCoding {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.huffman_encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.huffman_decode(data)
            .map_err(|e| anyhow!(DecompressionError::InvalidInput(e.to_string())))
    }
}

impl CompressorExt for HuffmanCoding {
    fn long_name(&self) -> &'static str {
        "Huffman Coding"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["huffman", "huff", "huffcode", "huffman_coding"]
    }
}

impl HuffmanCoding {
    pub fn huffman_encode(&mut self, data: &[u8]) -> Vec<u8> {
        todo!("Huffman coding is currently unimplemented")
    }

    pub fn huffman_decode(&mut self, data: &[u8]) -> Result<Vec<u8>, DecompressionError> {
        todo!("Huffman coding is currently unimplemented")
    }
}
