use crate::mutator::Mutator;
use anyhow::Result;
use voxell_timer::time_fn;

if_tracing! {
    use tracing::{info};
}

pub mod arcode;
pub mod bsc;
pub mod bwt;
pub mod huffman;
pub mod mtf;
pub mod pipeline;
pub mod re_pair;
pub mod serializing_algorithm;

#[derive(Clone, Copy, Debug)]
pub struct DynMutator {
    pub drive_mutation: fn(data: &[u8], buf: &mut Vec<u8>) -> Result<()>,
    pub revert_mutation: fn(data: &[u8], buf: &mut Vec<u8>) -> Result<()>,
}

impl Mutator for DynMutator {
    fn drive_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        let (res, d) = time_fn(|| (self.drive_mutation)(data, buf));
        if_tracing! {
            info!(out_len = buf.len(), "dyn drive_mutation finished in {:.1?}", d);
        }
        res
    }

    fn revert_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        let (r, d) = time_fn(|| (self.revert_mutation)(data, buf));
        if_tracing! {
            info!(out_len = buf.len(), "dyn revert_mutation finished in {:.1?}", d);
        }
        r
    }
}
