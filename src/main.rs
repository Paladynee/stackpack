#![feature(array_windows)]

#[cfg(test)]
mod tests;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;

use algorithms::arith::ArithmeticCoding;
use algorithms::bwt::Bwt;
use algorithms::mtf::Mtf;
use algorithms::pipeline::CompressionPipeline;
use algorithms::rle2::Rle2;
use algorithms::rle3::Rle3;
use compressor::Compressor;
use derive_fromstr::derive_fromstr;

extern crate arcode;
extern crate voxell_rng;

mod algorithms;
mod compressor;

#[derive_fromstr(trim, lowercase, truncate(3))]
#[derive(Debug, Clone, Copy, PartialEq)]
enum WorkingMode {
    Encode,
    Decode,
    Test,
    Corpus,
}

const USAGE: &str = concat!(
    "Usage:\n\t",
    env!("CARGO_PKG_NAME"),
    " enc <path> <output_path>\n\t",
    env!("CARGO_PKG_NAME"),
    " dec <path> <output_path>\n\t",
    env!("CARGO_PKG_NAME"),
    " test <original_file> <compressed_uncompressed_path>\n\t",
    env!("CARGO_PKG_NAME"),
    " corpus"
);

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", USAGE);
        return;
    }

    let working_mode: WorkingMode = args[1].as_str().parse().expect(USAGE);
    let mut pipeline = CompressionPipeline::new()
        // utilize all of the algorithms
        .with_algorithm(Bwt)
        .with_algorithm(Mtf)
        .with_algorithm(Rle3)
        .with_algorithm(ArithmeticCoding);

    if WorkingMode::Corpus == working_mode {
        // compress each file in ./test_data and save in ./test_data/file.vox
        let mut iife = || -> Result<(), io::Error> {
            let dir = "./test_data";
            let paths = fs::read_dir(dir)?;

            for path in paths {
                let path = path?.path();
                if path.extension().unwrap_or_default() == "vox" {
                    continue;
                }
                println!("Compressing file {:?}", path);
                if path.is_file() {
                    let mut output_path = path.clone();
                    output_path.set_extension("vox");

                    let file_data = fs::read(path)?;
                    let compressed_data = pipeline.compress_bytes(&file_data);
                    fs::write(output_path, compressed_data)?;
                }
            }
            Ok(())
        };
        if let Err(e) = iife() {
            println!("Error in corpus: {}", e);
        } else {
            println!("Corpus compression done");
        }
        return;
    }

    let inpath = args[2].as_str();
    let outpath = args[3].as_str();

    let Ok(meta) = fs::metadata(inpath) else {
        println!("Error: input path failure");
        return;
    };

    if !meta.is_file() {
        println!("Error: input path does not pass sanity checks");
        return;
    }

    let input_data = fs::read(inpath).expect("could not read the file into memory");

    // .with_algorithm(Box::new(Rle { debug: false }))
    // .with_algorithm(Box::new(Bwt { debug: false }))
    // .with_algorithm(Box::new(Mtf { debug: false }))
    // .with_algorithm(Box::new(ArithmeticCoding { debug: false }));

    let output_data = match working_mode {
        WorkingMode::Encode => pipeline.compress_bytes(&input_data),
        WorkingMode::Decode => pipeline.decompress_bytes(&input_data).expect("decompression error"),
        WorkingMode::Test => return file_sameness_test(input_data, outpath),
        WorkingMode::Corpus => unreachable!("Corpus mode has already been handled"),
    };

    fs::write(outpath, output_data).expect("could not write the file");
    println!("Done {}", if working_mode == WorkingMode::Encode { "encoding" } else { "decoding" });
}

fn file_sameness_test(data: Vec<u8>, path: &str) {
    // just checks whether the 2 files are the same bitwise.
    let original_file = data;
    let roundtrip_file = fs::read(path).expect("could not read the file into memory");
    if original_file == roundtrip_file {
        println!("Files are the same");
    } else {
        println!("Files are different");
    }
}
