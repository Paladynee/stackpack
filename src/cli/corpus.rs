use core::time::Duration;
use std::fs;
use std::path::Path;

use anyhow::Result;
use voxell_timer::time_fn;
use walkdir::WalkDir;

use crate::{
    cli::{CorpusArgs, PipelineSelection, pipeline},
    mutator::Mutator,
};

// add tracing imports
use tracing::{debug, error, info};

pub fn corpus(args: CorpusArgs) {
    run_folder(Path::new("./test_data"), args.pipeline_selection(), true);
}

pub fn run_folder(input_dir: &Path, selection: PipelineSelection, write_results: bool) {
    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() || e.file_type().is_symlink())
    {
        let path = entry.path();
        let mut pipeline = pipeline::build_pipeline(selection.clone());

        let input = fs::read(path).unwrap();
        let mut compressed = Vec::new();
        let (res, comp_dur) = time_fn(|| pipeline.drive_mutation(&input, &mut compressed));

        let mut decompressed = Vec::new();
        let (_, decomp_dur) = time_fn(|| pipeline.revert_mutation(&compressed, &mut decompressed));
        validate_and_print_results(
            res,
            path,
            &input[..],
            &compressed[..],
            &decompressed[..],
            comp_dur,
            decomp_dur,
            write_results,
        );
    }
}

fn save_failed_equality_results_to_file(expected: &[u8], intermediate: &[u8], got: &[u8], path: &Path) {
    let filename = path.file_name().unwrap().to_str().unwrap();
    let target_expected = format!("{}.expected.bin", filename);
    let target_intermediate = format!("{}.intermediate.bin", filename);
    let target_got = format!("{}.got.bin", filename);

    if fs::exists(&target_expected).unwrap() {
        fs::rename(&target_expected, format!("{}.old", &target_expected)).unwrap();
    };
    if fs::exists(&target_intermediate).unwrap() {
        fs::rename(&target_intermediate, format!("{}.old", &target_intermediate)).unwrap();
    };
    if fs::exists(&target_got).unwrap() {
        fs::rename(&target_got, format!("{}.old", &target_got)).unwrap();
    };

    fs::write(&target_expected, expected).unwrap();
    fs::write(&target_intermediate, intermediate).unwrap();
    fs::write(&target_got, got).unwrap();
}

#[allow(clippy::too_many_arguments)]
fn validate_and_print_results(
    res: Result<()>,
    path: &Path,
    expected: &[u8],
    intermediate: &[u8],
    got: &[u8],
    compression_time: Duration,
    decompression_time: Duration,
    write_results: bool,
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
    if !equality && write_results {
        save_failed_equality_results_to_file(expected, intermediate, got, path);
    }

    if_tracing! {
        info!("==== {} {} ====", passed_string, path.display());

        debug!(
            "encode: {:.0?}\ndecode: {:.0?}\noriginal: {} bytes\ncompressed: {} bytes\ndecompressed: {} bytes\nratio: {:.1}% (compressed/original)\nsaved: {:+} bytes ({:+.1}%)",
            compression_time,
            decompression_time,
            original_size,
            compressed_size,
            decompressed_size,
            ratio * 100.0,
            bytes_saved,
            percent_saved,
        );

        if !passed {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            let err_msg = res.as_ref().err().map(|e| e.to_string()).unwrap_or_else(|| "error".into());
            error!(
                "error: {}\nsee {}.expected.bin and {}.got.bin for details",
                err_msg, filename, filename
            );
        }
    };

    if_not_tracing! {
        eprintln!("{} {}", passed_string, path.display());
    }
}
