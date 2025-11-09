#![allow(unused)]
use std::fmt::Display;

//todo
use anyhow::{Result, anyhow};

use crate::algorithms::DynMutator;
pub const Huffman: DynMutator = DynMutator {
    drive_mutation: huffman_encode,
    revert_mutation: huffman_decode,
};

pub use self::Huffman as ThisMutator;

pub fn huffman_encode(_data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    todo!("Huffman coding is currently unimplemented")
}

pub fn huffman_decode(_data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    todo!("Huffman coding is currently unimplemented")
}
