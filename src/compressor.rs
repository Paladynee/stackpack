// TODO: refactor into stackpack-core.

pub use anyhow::Result;
use core::error::Error;

/// Represents an error emitted by the decompression algorithm while decoding data.
#[derive(Debug)]
pub enum DecompressionError {
    /// Input given to the decompressor was malformed, invalid, or otherwise incorrect for decoding.
    ///
    /// The argument is a string that describes what went wrong.
    InvalidInput(String),

    /// An internal error occurred in the decompressor. This should (practically) never happen.
    InternalDecoderError,
}

impl Error for DecompressionError {}

impl core::fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "Input data was malformed, and could not be decoded: {}", message),
            Self::InternalDecoderError => write!(f, "Internal decoder error occured."),
        }
    }
}

/// Represents shared behavior for all compressors.
///
/// Provides a method [`compress_bytes`] to compress data and [`decompress_bytes`] to decompress data.
///
/// # Note
///
/// No guarantees are made about the length of the resulting [`Vec<u8>`] from [`compress_bytes`].
/// It can be shorter, equal in length, or longer. The only guarantee is that [`decompress_bytes`]
/// will be able to reconstruct the original data.
///
/// [`compress_bytes`]: Compressor::compress_bytes
/// [`decompress_bytes`]: Compressor::decompress_bytes
pub trait Compressor: 'static {
    /// Compresses a given byte slice and returns the encoded data.
    ///
    /// Decoding the resulting [`Vec<u8>`] will always provide the original data.
    ///
    /// # Note
    ///
    /// No guarantees are made about the length of the resulting [`Vec<u8>`].
    /// It can be shorter, equal in length, or longer.
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8>;

    /// Decompresses a given byte slice and returns the decoded data.
    ///
    /// # Errors
    ///
    /// Returns an error if the input data was malformed, or if an internal decompressor error occurred.
    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>>;

    /// Returns the name of the compressor algorithm. Use for debugging purposes.
    ///
    /// Defaults to the type name of the compressor.
    fn compressor_name(&self) -> String {
        core::any::type_name::<Self>().to_string()
    }

    /// Performs a round-trip test on the compressor.
    ///
    /// Use for sanity checking the compressor and decompressor.
    fn test_roundtrip<'orig>(&mut self, data: &'orig [u8]) -> Result<RoundTripTestResult<'orig>> {
        let compressed = self.compress_bytes(data);
        let decompressed = self.decompress_bytes(&compressed)?;
        let equal = data == decompressed.as_slice();

        Ok(RoundTripTestResult {
            equal,
            original: data,
            compressed,
            decompressed,
        })
    }

    /// Converts the compressor into a boxed trait object.
    /// Useful for storing different compressors in a single collection.
    fn into_boxed(self) -> Box<dyn Compressor>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

/// Represents the result of a round-trip test.
///
/// Use accessor methods to retrieve the [`result`][RoundTripTestResult::is_successful],
/// the [`original data`][RoundTripTestResult::get_original],
/// the [`compressed data`][RoundTripTestResult::get_compressed],
/// and the [`decompressed data`][RoundTripTestResult::get_decompressed].
#[derive(Clone, Debug, Hash)]
pub struct RoundTripTestResult<'orig> {
    equal: bool,
    original: &'orig [u8],
    compressed: Vec<u8>,
    decompressed: Vec<u8>,
}

impl<'orig> RoundTripTestResult<'orig> {
    /// Whether the original and decompressed data were equal.
    pub const fn is_successful(&self) -> bool {
        self.equal
    }

    /// The original data before any action was taken.
    pub const fn get_original(&self) -> &'orig [u8] {
        self.original
    }

    /// The data after it has been encoded by the compressor.
    pub fn get_compressed(&self) -> &[u8] {
        self.compressed.as_slice()
    }

    /// The data after it has been decoded by the decompressor.
    pub fn get_decompressed(&self) -> &[u8] {
        self.decompressed.as_slice()
    }
}
