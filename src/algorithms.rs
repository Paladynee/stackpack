use crate::{mutator::Mutator, units::MEBIBYTES};
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
        if_tracing! {
            info!("data_len:MB" = data.len() as f64 / MEBIBYTES as f64, "dyn drive_mutation started");
        }
        let (res, d) = time_fn(|| (self.drive_mutation)(data, buf));
        if_tracing! {
            info!(
                out_len = buf.len(),
                ratio = data.len() as f64 / buf.len() as f64,
                "dyn drive_mutation finished in {:.1?}", d
            );
        }
        res
    }

    fn revert_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        if_tracing! {
            info!("data_len:MB" = data.len() as f64 / MEBIBYTES as f64, "dyn revert_mutation started");
        }
        let (r, d) = time_fn(|| (self.revert_mutation)(data, buf));
        if_tracing! {
            info!(
                out_len = buf.len(),
                ratio = data.len() as f64 / buf.len() as f64,
                "dyn revert_mutation finished in {:.1?}", d
            );
        }
        r
    }
}
