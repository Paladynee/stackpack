use anyhow::anyhow;

use crate::compressor::{Compressor, DecompressionError, Result};

pub struct RecursiveRle {
    pub debug: bool,
}

impl Compressor for RecursiveRle {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        rrle_encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        rrle_decode(data)
    }

    fn compressor_name(&self) -> String {
        "Recursive RLE".into()
    }
}

pub fn rrle_encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    // First pass: regular RLE
    let mut first_pass = Vec::new();
    let mut current_byte = data[0];
    let mut count: u8 = 1;

    for &byte in &data[1..] {
        if byte == current_byte && count < 255 {
            count += 1;
        } else {
            first_pass.push(count);
            first_pass.push(current_byte);
            current_byte = byte;
            count = 1;
        }
    }
    first_pass.push(count);
    first_pass.push(current_byte);

    // Second pass: compress the counts recursively
    let mut result = Vec::new();
    let mut i = 0;

    while i < first_pass.len() {
        if i % 2 == 0 {
            // Count value
            let count_value = first_pass[i];
            let mut run_length = 1;
            let mut j = i + 2;

            // Check for consecutive identical counts
            while j < first_pass.len() && first_pass[j] == count_value && run_length < 255 {
                run_length += 1;
                j += 2;
            }

            if run_length > 1 {
                // We found a run of identical counts
                result.push(0); // Marker for compressed counts
                result.push(run_length);
                result.push(count_value);

                // Add the byte values for each count in this run
                for k in (i..j).step_by(2) {
                    result.push(first_pass[k + 1]);
                }

                i = j;
            } else {
                // No compression for this count
                result.push(count_value);
                result.push(first_pass[i + 1]); // The byte value
                i += 2;
            }
        }
    }

    // Add a count of RLE passes
    let mut encoded = vec![1]; // 1 RLE pass left to undo
    encoded.append(&mut result);

    encoded
}

pub fn rrle_decode(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    // Read number of RLE passes left to undo
    let rle_passes = data[0];
    if rle_passes != 1 {
        return Err(anyhow!(DecompressionError::InvalidInput(format!(
            "Unsupported RLE pass count: {}",
            rle_passes
        ))));
    }

    // First, decode the compressed RLE format back to regular RLE
    let mut rle_data = Vec::new();
    let mut i = 1; // Skip the RLE pass count byte

    while i < data.len() {
        if data[i] == 0 {
            // This is a compressed count sequence
            if i + 2 >= data.len() {
                return Err(anyhow!(DecompressionError::InvalidInput("Truncated RRLE data".to_string())));
            }

            let run_length = data[i + 1] as usize;
            let count_value = data[i + 2];
            i += 3;

            // For each run, we need to read a byte value
            for _ in 0..run_length {
                if i >= data.len() {
                    return Err(anyhow!(DecompressionError::InvalidInput("Truncated RRLE data".to_string())));
                }
                rle_data.push(count_value); // Add the count
                rle_data.push(data[i]); // Add the byte value
                i += 1;
            }
        } else {
            // This is a regular count-byte pair
            if i + 1 >= data.len() {
                return Err(anyhow!(DecompressionError::InvalidInput("Truncated RRLE data".to_string())));
            }

            rle_data.push(data[i]); // Add the count
            rle_data.push(data[i + 1]); // Add the byte value
            i += 2;
        }
    }

    // Now expand the regular RLE
    let mut result = Vec::new();
    let mut j = 0;
    while j < rle_data.len() {
        if j + 1 >= rle_data.len() {
            return Err(anyhow!(DecompressionError::InvalidInput("Truncated RLE data".to_string())));
        }

        let count = rle_data[j];
        let byte = rle_data[j + 1];

        for _ in 0..count {
            result.push(byte);
        }

        j += 2;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_tests() {
        crate::tests::roundtrip_test(RecursiveRle { debug: false });
    }
}
