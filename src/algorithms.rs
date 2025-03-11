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