use std::fmt::Display;

use crate::compressor::{Compressor, CompressorExt, Result};

#[derive(Clone)]
pub struct Mtf;

impl Compressor for Mtf {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.mtf_encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.mtf_decode(data))
    }
}

impl Display for Mtf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Move-to-front Transform")
    }
}

impl CompressorExt for Mtf {
    fn aliases(&self) -> &'static [&'static str] {
        &["mtf", "move_to_front", "move_to_front_transform"]
    }

    fn dyn_clone(&self) -> Box<dyn CompressorExt> {
        Box::new(Self {})
    }
}

impl Mtf {
    pub fn mtf_encode(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut alphabet: Vec<u8> = (0..=255).collect();
        let mut encoded = Vec::with_capacity(data.len());

        for &symbol in data {
            let index = alphabet.iter().position(|&x| x == symbol).unwrap();
            encoded.push(index as u8);
            // alphabet.remove(index);
            // alphabet.insert(0, symbol);

            // use the specialized rotate_right to move the symbol to the front
            alphabet[..=index].rotate_right(1);
        }

        encoded
    }

    pub fn mtf_decode(&self, encoded: &[u8]) -> Vec<u8> {
        if encoded.is_empty() {
            return Vec::new();
        }

        let mut alphabet: Vec<u8> = (0..=255).collect();
        let mut decoded = Vec::with_capacity(encoded.len());

        for &index in encoded {
            let index = index as usize;
            let symbol = alphabet[index];

            decoded.push(symbol);
            alphabet.remove(index);
            alphabet.insert(0, symbol);
        }

        decoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_tests() {
        crate::tests::roundtrip_test(Mtf);
    }
}
