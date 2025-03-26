use core::cmp;
use core::fmt;
use core::fmt::Debug;
use core::str;
use std::fmt::Display;
use std::io::{self, Cursor, Read};

use anyhow::anyhow;

use crate::compressor::CompressorExt;
use crate::compressor::{Compressor, DecompressionError, Result};

#[derive(Clone)]
pub struct Rle {
    pub debug: bool,
}

impl Compressor for Rle {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.rle_encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.rle_decode(data)
            .map_err(|e| anyhow!(DecompressionError::InvalidInput(e.to_string())))
    }
}

impl Display for Rle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Run-length Encoding")
    }
}

impl CompressorExt for Rle {
    fn aliases(&self) -> &'static [&'static str] {
        &["rle", "run_length_encoding"]
    }

    fn dyn_clone(&self) -> Box<dyn CompressorExt> {
        Box::new(Self { debug: false })
    }
}

struct RleChunk {
    string_length: u8,
    /// this is `actual_repetitions - 1` so that we squeeze 1 more repetition since a 0 repetition is considered invalid.
    repetitions_minus_one: u8,
    string: Vec<u8>,
}

impl Debug for RleChunk {
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

impl RleChunk {
    fn get_size(&self) -> usize {
        2 + self.string.len()
    }
}

trait ReadRleChunk {
    fn read_rle_chunk(&mut self) -> Result<RleChunk, io::Error>;
}

impl<T: Read> ReadRleChunk for T {
    fn read_rle_chunk(&mut self) -> Result<RleChunk, io::Error> {
        let mut len: [u8; 1] = [0];
        self.read_exact(&mut len)?;
        let len = len[0];

        let mut repetitions: [u8; 1] = [0];
        self.read_exact(&mut repetitions)?;
        let repetitions = repetitions[0];

        let mut string = vec![0; len as usize];
        self.read_exact(&mut string)?;

        Ok(RleChunk {
            string_length: len,
            repetitions_minus_one: repetitions,
            string,
        })
    }
}

impl Rle {
    /// represents the given data using RLE chunks.
    fn chunkfinder(&self, data: &[u8]) -> Vec<RleChunk> {
        let mut chunks = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            // Find best chunk at current position
            let (chunk, advance) = self.find_optimal_chunk(data, pos);
            chunks.push(chunk);
            pos += advance;
        }

        chunks
    }

    pub fn rle_encode(&self, data: &[u8]) -> Vec<u8> {
        let chunks = self.chunkfinder(data);
        let size_guess = chunks.iter().fold(0, |mut acc, a| {
            acc += a.get_size();
            acc
        });

        let mut vec1 = Vec::with_capacity(size_guess);
        for chunk in chunks {
            if self.debug {
                println!("Encoded chunk: {:#?}", chunk);
            }
            vec1.push(chunk.string_length);
            vec1.push(chunk.repetitions_minus_one);
            vec1.extend(chunk.string);
        }
        vec1
    }

    fn find_optimal_chunk(&self, data: &[u8], start: usize) -> (RleChunk, usize) {
        // Option 1: Maximum non-repeating chunk
        let max_non_rep = cmp::min(255, data.len() - start);

        // Option 2: Try to find a repeating pattern
        let mut best_pattern_len = 0;
        let mut best_repeats = 0;
        let mut best_efficiency = 0.0;

        for pattern_len in 1..=cmp::min(255, data.len() - start) {
            let pattern = &data[start..(start + pattern_len)];
            // if pattern_len > 4 {
            //     // last 4 characters in pattern
            //     let last_four = &pattern[(pattern_len - 4)..];

            //     // eprintln!("{:?}", pattern);
            //     // check if all of them are the same
            //     if last_four.iter().all(|&x| x == last_four[0]) {
            //         // return an RleChunk that spans start..start+pattern_len-4
            //         return (
            //             RleChunk {
            //                 string_length: pattern_len as u8,
            //                 repetitions_minus_one: 0,
            //                 string: pattern[..(pattern_len - 4)].to_vec(),
            //             },
            //             pattern_len - 4,
            //         );
            //     }
            // } else if pattern_len == 4 {
            //     let last_four = &pattern[(pattern_len - 4)..];
            //     if last_four.iter().all(|&x| x == last_four[0]) {
            //         let character = last_four[0];
            //         // lookahead for the same characters until we find a different one, or encounter max_non_rep
            //         let mut repeats = 1;
            //         let mut pos = start;

            //         while pos < data.len() && data[pos] == character && repeats < 256 {
            //             repeats += 1;
            //             pos += 1;
            //         }

            //         return (
            //             RleChunk {
            //                 string_length: 1,
            //                 repetitions_minus_one: (repeats - 1) as u8,
            //                 string: vec![character],
            //             },
            //             repeats,
            //         );
            //     }
            // }

            let mut repeats = 1;
            let mut pos = start + pattern_len;

            while pos + pattern_len <= data.len() && &data[pos..(pos + pattern_len)] == pattern && repeats < 256 {
                repeats += 1;
                pos += pattern_len;
            }

            // calculate efficiency as compression ratio
            if repeats > 1 {
                let raw_size = pattern_len * repeats;
                let encoded_size = 2 + pattern_len; // 2 bytes overhead + pattern
                let efficiency = raw_size as f32 / encoded_size as f32;

                if efficiency > best_efficiency {
                    best_pattern_len = pattern_len;
                    best_repeats = repeats;
                    best_efficiency = efficiency;
                }
            }
        }

        // decide which option is better
        if best_efficiency > 1.0 {
            (
                RleChunk {
                    string_length: best_pattern_len as u8,
                    repetitions_minus_one: (best_repeats - 1) as u8,
                    string: data[start..(start + best_pattern_len)].to_vec(),
                },
                best_pattern_len * best_repeats,
            )
        } else {
            (
                RleChunk {
                    string_length: max_non_rep as u8,
                    repetitions_minus_one: 0,
                    string: data[start..(start + max_non_rep)].to_vec(),
                },
                max_non_rep,
            )
        }
    }

    /// decodes a list of RLE chunks into the data they represent.
    pub fn rle_decode(&self, data: &[u8]) -> Result<Vec<u8>> {
        fn dechunker(data: &[u8]) -> Result<Vec<RleChunk>> {
            let mut cursor = Cursor::new(data);
            let mut chunks = vec![];
            while cursor.position() < data.len() as u64 {
                let chunk = cursor.read_rle_chunk().map_err(|ioerr| {
                    anyhow!(DecompressionError::InvalidInput(format!(
                        "Failed to read RLE chunk: {}",
                        ioerr.to_string()
                    )))
                })?;
                chunks.push(chunk);
            }
            Ok(chunks)
        }

        if data.is_empty() {
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

            if self.debug {
                println!("Decoded chunk: {:#?}", chunk);
            }
        }

        Ok(vec1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_tests() {
        crate::tests::roundtrip_test(Rle { debug: false });
    }
}
