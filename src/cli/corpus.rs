use std::{fs, path::Path, time::Duration};

use anyhow::Result;
use voxell_timer::time_fn;
use walkdir::WalkDir;

use crate::{
    cli::{CorpusArgs, pipeline},
    mutator::Mutator,
};

pub fn corpus(_args: CorpusArgs) {
    let input_dir = "./test_data";
    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() || e.file_type().is_symlink())
    {
        let path = entry.path();
        let mut pipeline = pipeline::default_pipeline();

        let input = fs::read(path).unwrap();
        let mut compressed = Vec::new();
        let (res, comp_dur) = time_fn(|| pipeline.drive_mutation(&input, &mut compressed));

        let mut decompressed = Vec::new();
        let (_, decomp_dur) = time_fn(|| pipeline.revert_mutation(&compressed, &mut decompressed));
        validate_and_print_results(res, path, &input[..], &compressed[..], &decompressed[..], comp_dur, decomp_dur);
    }
}

fn save_failed_equality_results_to_file(expected: &[u8], got: &[u8], path: &Path) {
    let filename = path.file_name().unwrap().to_str().unwrap();
    let target_expected = format!("{}.expected.bin", filename);
    let target_got = format!("{}.got.bin", filename);

    if fs::exists(&target_expected).unwrap() {
        fs::rename(&target_expected, format!("{}.old", &target_expected)).unwrap();
    };
    if fs::exists(&target_got).unwrap() {
        fs::rename(&target_got, format!("{}.old", &target_got)).unwrap();
    };

    fs::write(&target_expected, expected).unwrap();
    fs::write(&target_got, got).unwrap();
}

fn validate_and_print_results(
    res: Result<()>,
    path: &Path,
    expected: &[u8],
    intermediate: &[u8],
    got: &[u8],
    compression_time: Duration,
    decompression_time: Duration,
) {
    let equality = expected == got;
    let original_size = expected.len();
    let compressed_size = intermediate.len();
    let decompressed_size = got.len();

    let ratio = if original_size == 0 {
        1.0
    } else {
        compressed_size as f64 / original_size as f64
    };

    let bytes_saved = original_size as isize - compressed_size as isize;
    let percent_saved = if original_size == 0 {
        0.0
    } else {
        (bytes_saved as f64) / (original_size as f64) * 100.0
    };

    let passed = equality && res.is_ok();

    let passed_string = if passed { "PASSED" } else { "FAILED" };
    if !equality {
        save_failed_equality_results_to_file(expected, got, path);
    }
    eprintln!(
        "======== {} {} ========\n\t{:.0?} encode\n\t{:.0?} decode\n\toriginal: {} bytes\n\tcompressed: {} bytes\n\tdecompressed: {} bytes\n\tratio: {:.1}% (compressed/original)\n\tsaved: {:+} bytes ({:+.1}%)\n\t{}",
        passed_string,
        path.display(),
        compression_time,
        decompression_time,
        original_size,
        compressed_size,
        decompressed_size,
        ratio * 100.0,
        bytes_saved,
        percent_saved,
        if !passed {
            format!(
                "error: {}\nsee {}.expected.bin and {}.got.bin for details",
                res.unwrap_err(),
                path.file_name().unwrap().to_str().unwrap(),
                path.file_name().unwrap().to_str().unwrap()
            )
        } else {
            String::new()
        }
    );
}
