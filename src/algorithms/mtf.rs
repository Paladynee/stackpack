use crate::compressor::{Compressor, Result};

pub struct Mtf;

impl Compressor for Mtf {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.mtf_encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.mtf_decode(data))
    }

    fn compressor_name(&self) -> String {
        "Move-to-Front Transform".into()
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
            alphabet.remove(index);
            alphabet.insert(0, symbol);
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
