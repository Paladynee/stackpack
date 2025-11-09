pub use anyhow::Result;
use anyhow::anyhow;
use core::{error::Error, fmt};

#[derive(Debug)]
pub enum DecompressionError {
    InvalidInput(String),
    InternalDecoderError,
}

impl Error for DecompressionError {}

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "Input data was malformed, and could not be decoded: {}", message),
            Self::InternalDecoderError => write!(f, "Internal decoder error occured."),
        }
    }
}

pub trait Compressor {
    fn compress_bytes(&mut self, data: &[u8], buf: &mut Vec<u8>);
    fn decompress_bytes(&mut self, data: &[u8], buf: &mut Vec<u8>) -> Result<()>;
    fn test_roundtrip<'orig>(&mut self, data: &'orig [u8]) -> Result<()> {
        let mut buf = vec![];
        <Self as Compressor>::compress_bytes(self, data, &mut buf);
        let mut decompress_buf = vec![];
        <Self as Compressor>::decompress_bytes(self, &buf, &mut decompress_buf)?;
        if buf == decompress_buf { Ok(()) } else { Err(anyhow!("not equal")) }
    }
}
