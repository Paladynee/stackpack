use core::cmp;
use core::fmt;
use core::fmt::Debug;
use core::str;
use std::io::{self, Cursor, Read};

use anyhow::anyhow;

use crate::compressor::{Compressor, DecompressionError, Result};

pub struct Rle2;

impl Compressor for Rle2 {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.rle_encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.rle_decode(data)
            .map_err(|e| anyhow!(DecompressionError::InvalidInput(e.to_string())))
    }

    fn compressor_name(&self) -> String {
        "Voxell's Run-Length Encoding".into()
    }
}

#[derive(Clone)]
pub struct RleChunk2 {
    string_length: u8,
    /// this is `actual_repetitions - 1` so that we squeeze 1 more repetition since a 0 repetition is considered invalid.
    repetitions_minus_one: u8,
    string: Vec<u8>,
}

impl Debug for RleChunk2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RleChunk")
            .field("string_length", &self.string_length)
            .field("repetitions", &(self.repetitions_minus_one as usize + 1))
            // the string as the hex representation
            .field(
                "string",
                &str::from_utf8(&self.string).map_or_else(|_| format!("Hex: {}", hexify(&self.string)), |s| format!("Text: {}", s)),
            )
            .finish()
    }
}

fn hexify(data: &[u8]) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    for byte in data {
        write!(s, "{:02x}", byte).unwrap();
    }
    s
}

impl RleChunk2 {
    fn get_size(&self) -> usize {
        2 + self.string.len()
    }
}

trait ReadRleChunk {
    fn read_rle_chunk(&mut self) -> Result<RleChunk2, io::Error>;
}

impl<T: Read> ReadRleChunk for T {
    fn read_rle_chunk(&mut self) -> Result<RleChunk2, io::Error> {
        let mut len: [u8; 1] = [0];
        self.read_exact(&mut len)?;
        let len = len[0];

        let mut repetitions: [u8; 1] = [0];
        self.read_exact(&mut repetitions)?;
        let repetitions = repetitions[0];

        let mut string = vec![0; len as usize];
        self.read_exact(&mut string)?;

        Ok(RleChunk2 {
            string_length: len,
            repetitions_minus_one: repetitions,
            string,
        })
    }
}

impl Rle2 {
    pub fn rle_encode(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 4 {
            return data.to_vec();
        }

        let mut chunks: Vec<RleChunk2> = vec![];

        let mut start = 0;
        while start < data.len() {
            let current = data[start];
            let mut count = 0;
            while start + count < data.len() && data[start + count] == current {
                count += 1;
            }
            // break long runs into multiple chunks if needed.
            let mut remaining = count;
            while remaining > 0 {
                let use_count = cmp::min(remaining, 256);
                chunks.push(RleChunk2 {
                    string_length: 1,
                    repetitions_minus_one: (use_count - 1) as u8,
                    string: vec![current],
                });
                remaining -= use_count;
            }
            start += count;
        }

        let mut best_size = chunks.iter().fold(0, |acc, chunk| acc + chunk.get_size());
        println!("this may take a while... best size: {}", best_size);
        loop {
            let candidate_chunks = aggregate_chunks(&chunks);
            let candidate_size = candidate_chunks.iter().fold(0, |acc, chunk| acc + chunk.get_size());
            if candidate_size >= best_size {
                break;
            }
            println!("new best size: {}", candidate_size);
            best_size = candidate_size;
            chunks = candidate_chunks;
        }

        let mut vec1: Vec<u8> = Vec::new();

        for chunk in chunks {
            vec1.push(chunk.string_length);
            vec1.push(chunk.repetitions_minus_one);
            vec1.extend_from_slice(&chunk.string);
        }

        vec1
    }

    /// decodes a list of RLE chunks into the data they represent.
    pub fn rle_decode(&self, data: &[u8]) -> Result<Vec<u8>, io::Error> {
        fn dechunker(data: &[u8]) -> Result<Vec<RleChunk2>, io::Error> {
            let mut cursor = Cursor::new(data);
            let mut chunks = vec![];
            while cursor.position() < data.len() as u64 {
                let chunk = cursor.read_rle_chunk()?;
                chunks.push(chunk);
            }
            Ok(chunks)
        }

        if data.len() < 4 {
            return Ok(Vec::new());
        }

        let chunks = dechunker(data)?;
        let size_guess = chunks.iter().fold(0, |mut acc, a| {
            acc += a.string.len() * (a.repetitions_minus_one as usize + 1);
            acc
        });

        let mut vec1 = Vec::with_capacity(size_guess);

        for chunk in chunks {
            for _ in 0..=chunk.repetitions_minus_one {
                vec1.extend(&chunk.string);
            }
        }

        Ok(vec1)
    }
}

/// Checks if two chunks can be joined and returns the joined chunk if possible
fn try_join_chunks(first: &RleChunk2, second: &RleChunk2) -> Option<RleChunk2> {
    // THE CASE ORDERS ARE IMPORTANT:
    // lets examine this example
    // len: 1, reps: 0, string: [0x01]
    // len: 1, reps: 0, string: [0x02]
    // len: 1, reps: 0, string: [0x01]
    // len: 1, reps: 0, string: [0x02]
    //
    // if case 2 were applied first, it would join the entire thing into a raw form:
    // len: 4, reps: 0, string: [0x01, 0x02, 0x01, 0x02]
    // which is not optimal, since it can be represented using a single chunk by following these steps:
    //
    // step1(using case 1):
    // len: 1, reps: 0, string: [0x01, 0x02]
    // len: 1, reps: 0, string: [0x01, 0x02]
    //
    // step2(using case 2):
    // len: 2, reps: 0, string: [0x01, 0x02]

    // Case 1: Raw string consolidation (both have repetitions_minus_one = 0)
    if first.repetitions_minus_one == 0 && second.repetitions_minus_one == 0 {
        let combined_len = first.string.len() + second.string.len();
        if combined_len <= 255 {
            // combined chunk is more efficient if its size < sum of individual chunk sizes
            let combined_size = 2 + combined_len; // header + string length
            let individual_size = first.get_size() + second.get_size();

            if combined_size < individual_size {
                let mut combined_string = first.string.clone();
                combined_string.extend_from_slice(&second.string);

                return Some(RleChunk2 {
                    string_length: combined_len as u8,
                    repetitions_minus_one: 0,
                    string: combined_string,
                });
            }
        }
    }

    // Case 2: Same string consolidation
    if first.string == second.string && (first.repetitions_minus_one as u16 + second.repetitions_minus_one as u16) < 255 {
        let total_reps = first.repetitions_minus_one as u16 + second.repetitions_minus_one as u16 + 1;
        return Some(RleChunk2 {
            string_length: first.string_length,
            repetitions_minus_one: total_reps as u8,
            string: first.string.clone(),
        });
    }

    // Case 3: Mix of repeated and non-repeated (first has repetitions, second doesn't)
    if first.repetitions_minus_one > 0 && second.repetitions_minus_one == 0 {
        // check if converting to raw string would be more efficient
        let mut raw_string = Vec::new();
        for _ in 0..=first.repetitions_minus_one {
            raw_string.extend_from_slice(&first.string);
        }
        raw_string.extend_from_slice(&second.string);

        if raw_string.len() <= 255 {
            let combined_size = 2 + raw_string.len(); // header + string length
            let individual_size = first.get_size() + second.get_size();

            if combined_size < individual_size {
                return Some(RleChunk2 {
                    string_length: raw_string.len() as u8,
                    repetitions_minus_one: 0,
                    string: raw_string,
                });
            }
        }
    }

    None // Cannot join
}

/// aggregates sequential short chunks that can be represented using the "raw string" mode,
/// with `repetitions_minus_one` set to 0 (occurs only once in decoded stream)
pub fn aggregate_chunks(chunks: &[RleChunk2]) -> Vec<RleChunk2> {
    if chunks.is_empty() {
        return vec![];
    }

    let mut result = Vec::with_capacity(chunks.len());
    let mut i = 0;

    while i < chunks.len() {
        let mut current = chunks[i].clone();
        let mut j = i + 1;

        while j < chunks.len() {
            if let Some(joined) = try_join_chunks(&current, &chunks[j]) {
                current = joined;
                j += 1;
            } else {
                break;
            }
        }

        result.push(current);
        i = j;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_tests() {
        crate::tests::roundtrip_test(Rle2);
    }
}
