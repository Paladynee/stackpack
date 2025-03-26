use std::fmt::Display;

use anyhow::anyhow;

use crate::compressor::{Compressor, CompressorExt, DecompressionError, Result};

#[derive(Clone)]
pub struct Rle3;

impl Compressor for Rle3 {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        decode(data)
    }
}

impl Display for Rle3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Run-Length Encoding 3")
    }
}

impl CompressorExt for Rle3 {
    fn aliases(&self) -> &'static [&'static str] {
        &["rle3", "run_length_encoding_3"]
    }

    fn dyn_clone(&self) -> Box<dyn CompressorExt> {
        Box::new(Self {})
    }
}

fn encode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut literal_buf = Vec::new();
    let mut i = 0;

    while i < input.len() {
        // Look for a beneficial repeated chunk starting at i.
        if let Some((chunk_len, count, _saving)) = detect_best_run(input, i) {
            // Flush any pending literal bytes first.
            if !literal_buf.is_empty() {
                flush_literal(&mut output, &mut literal_buf);
            }
            let chunk = &input[i..i + chunk_len];
            let mut remaining = count;
            // If the repetition count exceeds 255, we break it into multiple chunks.
            while remaining > 0 {
                let cur_count = remaining.min(255);
                output.push(chunk_len as u8);
                output.push(cur_count as u8);
                output.extend_from_slice(chunk);
                remaining -= cur_count;
            }
            i += chunk_len * count;
        } else {
            // No beneficial repetition: add current byte to literal buffer.
            literal_buf.push(input[i]);
            i += 1;
            // Flush literal if it reaches the maximum allowed chunk size.
            if literal_buf.len() == 255 {
                flush_literal(&mut output, &mut literal_buf);
            }
        }
    }

    if !literal_buf.is_empty() {
        flush_literal(&mut output, &mut literal_buf);
    }
    output
}

fn decode(encoded: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut i = 0;
    while i < encoded.len() {
        if i + 2 > encoded.len() {
            return Err(anyhow!(DecompressionError::InvalidInput("Incomplete header at end of input".to_string())));
        }
        let chunk_len = encoded[i] as usize;
        let count = encoded[i + 1] as usize;
        i += 2;
        if i + chunk_len > encoded.len() {
            return Err(anyhow!(DecompressionError::InvalidInput("Incomplete chunk data".to_string())));
        }
        let chunk = &encoded[i..i + chunk_len];
        for _ in 0..count {
            output.extend_from_slice(chunk);
        }
        i += chunk_len;
    }
    Ok(output)
}

fn detect_best_run(input: &[u8], i: usize) -> Option<(usize, usize, usize)> {
    let max_possible_chunk = (input.len() - i).min(255);
    let mut best: Option<(usize, usize, usize)> = None;

    for chunk_len in 1..=max_possible_chunk {
        let mut count = 1;
        while i + chunk_len * (count + 1) <= input.len() && input[i..i + chunk_len] == input[i + chunk_len * count..i + chunk_len * (count + 1)] {
            count += 1;
        }
        if count > 1 {
            let saving = chunk_len * (count - 1) - 2;
            if saving > 0 {
                best = match best {
                    Some((_, _, best_saving)) if saving > best_saving => Some((chunk_len, count, saving)),
                    None => Some((chunk_len, count, saving)),
                    _ => best,
                };
            }
        }
    }
    best
}

fn flush_literal(output: &mut Vec<u8>, literal: &mut Vec<u8>) {
    let mut start = 0;
    while start < literal.len() {
        let end = (start + 255).min(literal.len());
        let chunk = &literal[start..end];
        output.push(chunk.len() as u8); // chunk length
        output.push(1u8); // repetition count 1 indicates literal
        output.extend_from_slice(chunk);
        start = end;
    }
    literal.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_tests() {
        crate::tests::roundtrip_test(Rle3);
    }
}
