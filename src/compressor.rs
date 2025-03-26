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
            Self::InvalidInput(message) => {
                write!(f, "Input data was malformed, and could not be decoded: {}", message)
            }
            Self::InternalDecoderError => {
                write!(f, "Internal decoder error occured.")
            }
        }
    }
}

/// Represents shared behavior for all compressors.
///
/// Provides a method [`compress_bytes`](Compressor::compress_bytes) to compress data and
/// [`decompress_bytes`](Compressor::decompress_bytes) to decompress data.
///
/// # Note
///
/// No guarantees are made about the length of the resulting [`Vec<u8>`] from
/// [`compress_bytes`](Compressor::compress_bytes). It can be shorter, equal in length, or longer.
/// The only guarantee is that [`decompress_bytes`](Compressor::decompress_bytes) will be able to
/// reconstruct the original data.
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

    /// Performs a round-trip test on the compressor.
    ///
    /// Use for sanity checking the compressor and decompressor.
    fn test_roundtrip<'orig>(&mut self, data: &'orig [u8]) -> Result<RoundTripTestResult<'orig>> {
        let compressed = <Self as Compressor>::compress_bytes(self, data);
        let decompressed = <Self as Compressor>::decompress_bytes(self, &compressed)?;
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

pub trait CompressorExt: Compressor {
    /// Checks whether the input data is in a correct format that the decompressor can handle.
    /// This does not mean the decompressor is guaranteed to decode the data.
    ///
    /// This should be implemented as lazily as possible. No decoding should actually take place,
    /// only checking whether the format(layout of the underlying bytes) is valid for the decompressor.
    fn format_validity_check(&mut self, data: &[u8]) -> Result<bool> {
        // greedy implementation, because `decompress_bytes` will fail either way.
        Ok(true)
    }

    /// Returns the name of the compressor algorithm.
    /// Use for debugging purposes.
    ///
    /// Defaults to the type name of the compressor.
    fn debug_name(&self) -> String {
        core::any::type_name::<Self>().to_string()
    }

    /// Returns the canonical aliases of the compressor.
    /// Used for parsing the algorithm from strings.
    ///
    /// The aliases should be ordered by priority, with the most common alias first.
    ///
    /// Must be all-lowercased, and can only use characters from `a-zA-Z0-9_`.
    fn aliases(&self) -> &'static [&'static str];

    /// Clone the given compressor, returning an owned trait object.
    ///
    /// This must be implemented so that the compressor can be re-constructed from, say, a string.
    /// The returned compressor should be of default state, in other words, a compressor utilizing inner
    /// state should not include its state in the new compressor.
    fn dyn_clone(&self) -> Box<dyn CompressorExt>;
}

/// Represents the result of a round-trip test.
///
/// Use accessor methods to retrieve the [`result`][RoundTripTestResult::is_successful],
/// the [`original data`][RoundTripTestResult::get_original],
/// the [`compressed data`][RoundTripTestResult::get_compressed],
/// and the [`decompressed data`][RoundTripTestResult::get_decompressed].
#[derive(Clone, Debug, Hash)]
pub struct RoundTripTestResult<'orig> {
    pub(crate) equal: bool,
    pub(crate) original: &'orig [u8],
    pub(crate) compressed: Vec<u8>,
    pub(crate) decompressed: Vec<u8>,
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
