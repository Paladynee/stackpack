pub use anyhow::Result;

pub trait Mutator {
    fn drive_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()>;
    fn revert_mutation(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()>;
}
