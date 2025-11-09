use crate::cli::{EncodeArgs, pipeline};
use crate::compressor::Compressor;
use std::fs;
use voxell_timer::time_fn;

pub fn encode(args: EncodeArgs) {
    let input_path = &args.input;
    let output_path = &args.output;
    let mut pipeline = pipeline::build_pipeline(args.pipeline_selection());

    let input_data = fs::read(input_path).expect("Failed to read input file");
    let mut compressed_data = Vec::new();
    let ((), comp_dur) = time_fn(|| pipeline.compress_bytes(&input_data, &mut compressed_data));
    if_tracing! {
        tracing::info!(event = "encode_complete", input = %input_path.display(), output = %output_path.display(), elapsed_ms = %comp_dur.as_micros(), compressed_len = compressed_data.len(), "encode finished");
    }
    fs::write(output_path, compressed_data).expect("Failed to write output file");
}
