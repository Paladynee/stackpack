use core::cmp::Ordering;

use anyhow::anyhow;

use crate::compressor::{Compressor, DecompressionError, Result};

pub struct Bwt;

fn cmp_rotations(data: &[u8], i: usize, j: usize) -> Ordering {
    let len = data.len();
    for k in 0..len {
        let a = data[(i + k) % len];
        let b = data[(j + k) % len];
        if a != b {
            return a.cmp(&b);
        }
    }
    Ordering::Equal
}

impl Compressor for Bwt {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.bwt_encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.bwt_decode(data)
            .map_err(|e| anyhow!(DecompressionError::InvalidInput(format!("bwt decoder error: {}", e))))
    }

    fn compressor_name(&self) -> String {
        "Burrows-Wheeler Transform".into()
    }
}

impl Bwt {
    fn bwt_encode(&mut self, data: &[u8]) -> Vec<u8> {
        // If the input is too short, just return the original data.
        if data.len() < 4 {
            return data.to_vec();
        }

        // If input is too large, return original data
        if data.len() > u32::MAX as usize {
            return data.to_vec();
        }

        let n = data.len();
        // Create a vector of rotation starting indices.
        let mut rotations: Vec<usize> = (0..n).collect();

        // Sort rotations lexicographically using our custom comparator.
        rotations.sort_by(|&i, &j| cmp_rotations(data, i, j));

        // Find the index of the original string (rotation starting at 0)
        let orig_index = rotations.iter().position(|&i| i == 0).expect("Rotation starting at 0 must exist");

        // Build the transformed output: the last column of each rotation.
        // For a rotation starting at index i, the last character is at index (i + n - 1) % n.
        let mut bwt_transformed = Vec::with_capacity(n);
        for &rot in &rotations {
            let last_char = data[(rot + n - 1) % n];
            bwt_transformed.push(last_char);
        }

        // Prepend the original index (as 4 bytes, little-endian) to the transformed data.
        let mut output = Vec::with_capacity(4 + n);
        output.extend_from_slice(&(orig_index as u32).to_le_bytes());
        output.extend_from_slice(&bwt_transformed);
        output
    }

    #[allow(clippy::missing_asserts_for_indexing)]
    fn bwt_decode(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        // return the original data if the input is shorter than 4 bytes
        if data.len() < 4 {
            return Ok(data.to_vec());
        }

        // Read the original index (first 4 bytes, little-endian).
        let orig_index = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let bwt_transformed = &data[4..];
        let n = bwt_transformed.len();

        // If there's no actual data after the index, return empty
        if n == 0 {
            return Ok(Vec::new());
        }

        // Validate that orig_index is within bounds
        if orig_index >= n {
            return Err(format!("Invalid original index: {} (data length: {})", orig_index, n));
        }

        // Build the frequency table for each byte.
        let mut freq = [0usize; 256];
        for &byte in bwt_transformed {
            freq[byte as usize] += 1;
        }

        // Compute the starting position for each byte in the sorted first column.
        let mut starts = [0usize; 256];
        let mut sum = 0;
        for b in 0..256 {
            starts[b] = sum;
            sum += freq[b];
        }

        // Build the LF-mapping: lf[i] gives the next row in the reconstruction.
        let mut lf = vec![0usize; n];
        let mut seen = [0usize; 256];
        for (i, &byte) in bwt_transformed.iter().enumerate() {
            lf[i] = starts[byte as usize] + seen[byte as usize];
            seen[byte as usize] += 1;
        }

        // Reconstruct the original string using the LF mapping.
        let mut result = vec![0u8; n];
        let mut row = orig_index;
        // Reconstruct in reverse order.
        for i in (0..n).rev() {
            result[i] = bwt_transformed[row];
            row = lf[row];
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_tests() {
        crate::tests::roundtrip_test(Bwt);
    }
}
