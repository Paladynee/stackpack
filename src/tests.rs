#![allow(unused)]
use voxell_rng::rng::XorShift128;

use crate::compressor::Compressor;

const SHORT_DATA: &[u8] = b"Hello, World!";
const LONG_DATA: &[u8] =
    b"This is a longer string to test the arithmetic coding algorithm. It should be able to handle various lengths and characters.";
// FIXME: shim
// const LONGEST_DATA: &[u8] = include_bytes!("../test_data/enwik18793.txt");
const LONGEST_DATA: &[u8] = "shim";
// FIXME: shim
// const BINARY_DATA: &[u8] = include_bytes!("../test_data/CommonServiceLocator.dll");
const BINARY_DATA: &[u8] = "shim";
const RNG_DATA: &[u8] = &const {
    // (0..1000).map(|_| XorShift128::default().next_u8()).collect();
    let mut arr = [0u8; 1000];
    let mut rng = XorShift128::new(0xdeadcafe);
    let mut i = 0;
    while i < 1000 {
        let data = rng.peek_next_u64();
        arr[i] = (data & 0xFF) as u8;
        rng = XorShift128::new(data);
        i += 1;
    }
    arr
};
const REPEATING_DATA: &[u8] = b"a baba da babble da dabble babble doo bee babble dabble dooble dee boo dooble daddle boo";
const EMPTY_DATA: &[u8] = &[];
// TODO: add more test cases
// possibly utilizing some real corpus data

const TEST_CASES: &[(&[u8], &str)] = &[
    (REPEATING_DATA, "repeating data"),
    (SHORT_DATA, "short data"),
    (LONG_DATA, "long data"),
    (LONGEST_DATA, "longest data"),
    (BINARY_DATA, "binary data"),
    (RNG_DATA, "rng data"),
    (EMPTY_DATA, "empty data"),
];

pub fn roundtrip_test<C: Compressor>(mut compressor: C) {
    for &(test_case, test_name) in TEST_CASES {
        match compressor.test_roundtrip(test_case) {
            Ok(eq) => {
                let ratio = compression_ratio(eq.get_original(), eq.get_compressed());

                eprintln!(
                    "Compression ratio for {} with {}: {:.2}%",
                    test_name,
                    compressor.compressor_name(),
                    ratio * 100.0
                );

                assert!(
                    eq.is_successful(),
                    "Roundtrip test for {} failed at {}:\n\tExpected: {:?}\n\tGot: {:?}\n\tCompressed: {:?}",
                    compressor.compressor_name(),
                    test_name,
                    eq.get_original(),
                    eq.get_decompressed(),
                    eq.get_compressed(),
                );
            }
            Err(e) => {
                panic!(
                    "Fatal error while trying to compress/decompress {} with {}: {}",
                    test_name,
                    compressor.compressor_name(),
                    e
                );
            }
        }
    }
}

pub fn compression_ratio(original: &[u8], compressed: &[u8]) -> f64 {
    if original.is_empty() {
        return 0.0;
    }
    let ratio = compressed.len() as f64 / original.len() as f64;
    ratio
}
