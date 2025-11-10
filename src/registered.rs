use std::sync::LazyLock;

use anyhow::Result;
use parking_lot::Mutex;

use crate::{
    algorithms::{DynMutator, arcode, bsc, bwt, mtf, re_pair},
    mutator::Mutator,
    plugins::FfiMutator,
};

#[derive(Debug, Clone)]
pub enum EnumMutator {
    Dyn(DynMutator),
    Ffi(FfiMutator),
}

#[derive(Debug, Clone)]
pub struct RegisteredCompressor {
    pub mutator: EnumMutator,
    pub name: &'static str,
    pub short_description: Option<&'static str>,
}

impl RegisteredCompressor {
    pub const fn new_dyn(mutator: DynMutator, name: &'static str, short_description: Option<&'static str>) -> Self {
        RegisteredCompressor {
            mutator: EnumMutator::Dyn(mutator),
            name,
            short_description,
        }
    }

    pub const fn new_ffi(mutator: FfiMutator, name: &'static str, short_description: Option<&'static str>) -> Self {
        RegisteredCompressor {
            mutator: EnumMutator::Ffi(mutator),
            name,
            short_description,
        }
    }
}

/// Algorithms that are available to stackpack, and ones that are loaded at runtime.
pub static ALL_COMPRESSORS: LazyLock<Mutex<Vec<RegisteredCompressor>>> =
    LazyLock::new(|| Mutex::new(vec![arcode::ArithmeticCoding, bwt::Bwt, mtf::Mtf, bsc::Bsc, re_pair::RePair]));

impl Mutator for RegisteredCompressor {
    fn drive_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            let span = tracing::span!(tracing::Level::DEBUG, "registered compressor", name = self.name);
            let _span = span.enter();
            let res = match self.mutator {
                EnumMutator::Dyn(m) => (m.drive_mutation)(data, buf),
                EnumMutator::Ffi(ref mut m) => m.drive_mutation(data, buf),
            };
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
            let res = match self.mutator {
                EnumMutator::Dyn(m) => (m.drive_mutation)(data, buf),
                EnumMutator::Ffi(ref mut m) => m.drive_mutation(data, buf),
            };
            drop(_span);
            res
        }
        if_not_tracing! {
            (self.mutator.revert_mutation)(data, buf)
        }
    }
}
