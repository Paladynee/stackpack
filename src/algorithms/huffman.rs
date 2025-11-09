#![allow(unused)]
use std::fmt::Display;

//todo
use anyhow::{Result, anyhow};

use crate::{algorithms::DynCompressor, compressor::DecompressionError};
pub const Huffman: DynCompressor = DynCompressor {
    compress: huffman_encode,
    decompress: huffman_decode,
};

pub use self::Huffman as ThisCompressor;

pub fn huffman_encode(_data: &[u8], buf: &mut Vec<u8>) {
    todo!("Huffman coding is currently unimplemented")
}

pub fn huffman_decode(_data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    todo!("Huffman coding is currently unimplemented")
}
