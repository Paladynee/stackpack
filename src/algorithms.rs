use crate::compressor::CompressorExt;

/// Arithmetic coding, provided by the [arcode] crate.
pub mod arith;
/// The burrows-wheeler transform. Currently unoptimized.
pub mod bwt;
/// Huffman coding. Unimplemented.
pub mod huffman;
/// Move to front transform. I have no idea whether it is optimized or not.
pub mod mtf;
/// Recursive pairing compression. Unimplemented.
pub mod re_pair;

// the rle family of compressors, in my attempt to implement more than one
pub mod recursive_rle;
pub mod rle;
pub mod rle2;
pub mod rle3;

// a compression pipeline that combines multiple algorithms
pub mod pipeline;

/// All algorithms available in the current build of stackpack.
pub const ALL_COMPRESSORS: [&'static dyn CompressorExt; 8] = [
    &arith::ArithmeticCoding,
    &bwt::Bwt,
    &mtf::Mtf,
    &re_pair::RePair { debug: false },
    &rle::Rle { debug: false },
    &rle2::Rle2,
    &rle3::Rle3,
    &recursive_rle::RecursiveRle { debug: false },
];

pub const fn all() -> &'static [&'static dyn CompressorExt] {
    &ALL_COMPRESSORS
}
