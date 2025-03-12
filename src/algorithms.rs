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
pub fn all() -> Vec<&'static dyn CompressorExt> {
    let mut compressors: Vec<&dyn CompressorExt> = vec![];
    compressors.push(&arith::ArithmeticCoding);
    compressors.push(&bwt::Bwt);
    compressors.push(&mtf::Mtf);
    compressors.push(&re_pair::RePair { debug: false });
    compressors.push(&rle::Rle { debug: false });
    compressors.push(&rle2::Rle2);
    compressors.push(&rle3::Rle3);
    compressors.push(&recursive_rle::RecursiveRle { debug: false });
    compressors
}
