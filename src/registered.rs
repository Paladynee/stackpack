use anyhow::Result;

use crate::{
    algorithms::{DynMutator, arcode, bsc, bwt, mtf, re_pair},
    mutator::Mutator,
};

#[derive(Debug, Clone)]
pub struct RegisteredCompressor {
    pub mutator: DynMutator,
    pub name: &'static str,
}

impl RegisteredCompressor {
    const fn new(mutator: DynMutator, name: &'static str) -> Self {
        Self { mutator, name }
    }
}

/// All algorithms available in the current build of stackpack.
#[rustfmt::skip]
pub static ALL_COMPRESSORS: &[RegisteredCompressor] = &[
    RegisteredCompressor::new(arcode::ThisMutator, "arcode"),
    RegisteredCompressor::new(bwt::ThisMutator, "bwt"),
    RegisteredCompressor::new(mtf::ThisMutator, "mtf"),
    RegisteredCompressor::new(bsc::ThisMutator, "bsc"),
    RegisteredCompressor::new(re_pair::ThisMutator, "re_pair"),
];

impl Mutator for RegisteredCompressor {
    fn drive_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let span = tracing::span!(tracing::Level::DEBUG, "registered compressor", name = self.name);
            let _span = span.enter();
            let res = (self.mutator.drive_mutation)(data, buf);
            drop(_span);
            res
        }
        if_not_tracing! {
            (self.mutator.drive_mutation)(data, buf)
        }
    }

    fn revert_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let span = tracing::span!(tracing::Level::DEBUG, "registered decompressor", name = self.name);
            let _span = span.enter();
            let res = (self.mutator.revert_mutation)(data, buf);
            drop(_span);
            res
        }
        if_not_tracing! {
            (self.mutator.revert_mutation)(data, buf)
        }
    }
}
