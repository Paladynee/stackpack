#![allow(unused)] //todo
use core::fmt;
use core::fmt::{Debug, Display};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hasher},
};

use anyhow::Result;

use crate::algorithms::DynMutator;

pub const RePair: DynMutator = DynMutator {
    drive_mutation: repair_encode,
    revert_mutation: repair_decode,
};

pub use self::RePair as ThisMutator;

/// when any value of this type is <= 255, it stores a value as-is.
/// otherwise, it points to another entry in the grammar, using itself as an index.
type GrammarIndexOrRawByte = u32;

#[derive(Hash, Clone, PartialEq, Eq)]
pub enum Symbol {
    Long { data: GrammarIndexOrRawByte, len: usize },
    Short(GrammarIndexOrRawByte),
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Long { data, len } => match data {
                a @ 0..=255 if (*a as u8).is_ascii() => f.write_str(format!("{} repeating {} times", (*data as u8) as char, len).as_str()),
                _ => f.debug_struct("Long").field("data", data).field("len", len).finish(),
            },
            Symbol::Short(data) => match data {
                a @ 0..=255 if (*a as u8).is_ascii() => f.write_str(format!("{}", (*data as u8) as char).as_str()),
                _ => f.debug_struct("Short").field("data", data).finish(),
            },
        }
    }
}

#[derive(Clone)]
pub struct Grammar {
    inner: Vec<u32>,
}

pub fn repair_encode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    let initial_values = (0u32..=255u32).collect::<Vec<_>>();
    let mut grammar = Grammar { inner: initial_values };
    let mut charlist = data.iter().map(|&byte| Symbol::Short(u32::from(byte))).collect::<Vec<_>>();
    let mut frequencies: HashMap<&[Symbol], usize> = HashMap::new();

    for window in charlist.windows(2) {
        let entry = frequencies.entry(window).or_insert(0);
        *entry += 1;
    }

    todo!()
}

pub fn repair_decode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    todo!("{:?}", data.to_vec());
}
