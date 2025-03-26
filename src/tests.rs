use std::fmt::Display;

use anyhow::Result;
use voxell_rng::rng::XorShift128;

use crate::compressor::{Compressor, CompressorExt, DecompressionError, RoundTripTestResult};

const SHORT_TEXT: &[u8] =
    b"Hello, Entropy! This is a short data string to test small inputs. It has little repetition, but it should still be handled correctly.";
const MID_TEXT: &[u8] = b"This is a longer string to test the compression algorithm. It should be able to handle various lengths and characters. \
    We need to ensure that our compression algorithms work efficiently with different kinds of text patterns, including repeated words, \
    special characters like !@#$%^&*(), numbers 0123456789, and varying sentence structures. The longer the test data, the more likely \
    we are to catch edge cases in our implementation.";
const LONG_TEXT: &[u8] = b"";
const EMPTY_DATA: &[u8] = &[];
// .tar taken from one of my modules that i dont remember the name of
const TAR_DATA: &[u8] = include_bytes!("../test_data/node_modules.tar");
// .dll taken from TEdit terraria map editor v4.16.1 https://github.com/TEdit/Terraria-Map-Editor/releases/tag/4.16.1
const DLL_DATA: &[u8] = include_bytes!("../test_data/Newtonsoft.Json.dll");
// a PE executable of one of the earlier versions of this crate, with some bytes changed to make it not-executable for obvious reasons
const EXECUTABLE_DATA: &[u8] = include_bytes!("../test_data/stackpack.exe");
// .json taken from TEdit v4.16.1 (again) https://github.com/TEdit/Terraria-Map-Editor/releases/tag/4.16.1
const JSON_DATA: &[u8] = include_bytes!("../test_data/npcData.json");

// TODO: add more test cases
// possibly utilizing some real corpus data

const TEST_CASES: &[(&[u8], &str)] = &[
    (SHORT_TEXT, "Short Text"),
    (MID_TEXT, "Medium Text"),
    (LONG_TEXT, "Long Text"),
    (EMPTY_DATA, "Empty Data"),
    (TAR_DATA, "TAR Data"),
    (DLL_DATA, "DLL Data"),
    (EXECUTABLE_DATA, "Executable Data"),
    (JSON_DATA, "JSON Data"),
];

pub fn roundtrip_test<C: CompressorExt + Display>(mut compressor: C) {
    for &(test_case, test_name) in TEST_CASES {
        match compressor.test_roundtrip(test_case) {
            Ok(eq) => {
                let ratio = compression_ratio(eq.get_original(), eq.get_compressed());

                eprintln!(
                    "Compression ratio for {: >15} with {}: (comp/orig){:.2}%",
                    test_name,
                    compressor,
                    ratio * 100.0
                );

                assert!(
                    eq.is_successful(),
                    "Roundtrip test for {} failed at {}:\n\tExpected: {:?}\n\tGot: {:?}\n\tCompressed: {:?}",
                    compressor,
                    test_name,
                    eq.get_original(),
                    eq.get_decompressed(),
                    eq.get_compressed(),
                );
            }
            Err(e) => {
                panic!("Fatal error while trying to compress/decompress {} with {}: {}", test_name, compressor, e);
            }
        }
    }
}

pub fn roundtrip_test_basic_compressor<C: Compressor>(mut compressor: C, compressor_name: &str) {
    for &(test_case, test_name) in TEST_CASES {
        let mut f = || -> Result<RoundTripTestResult> {
            let compressed = compressor.compress_bytes(test_case);
            let decompressed = compressor.decompress_bytes(&compressed)?;
            let equal = test_case == decompressed.as_slice();

            Ok(RoundTripTestResult {
                equal,
                original: test_case,
                compressed,
                decompressed,
            })
        };

        let roundtrip = f();

        match roundtrip {
            Ok(eq) => {
                let ratio = compression_ratio(eq.get_original(), eq.get_compressed());

                eprintln!("Compression ratio for {} with {}: {:.2}%", test_name, compressor_name, ratio * 100.0);

                assert!(
                    eq.is_successful(),
                    "Roundtrip test for {} failed at {}:\n\tExpected: {:?}\n\tGot: {:?}\n\tCompressed: {:?}",
                    compressor_name,
                    test_name,
                    eq.get_original(),
                    eq.get_decompressed(),
                    eq.get_compressed(),
                );
            }
            Err(e) => {
                panic!(
                    "Fatal error while trying to compress/decompress {} with {}: {}",
                    test_name, compressor_name, e
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
