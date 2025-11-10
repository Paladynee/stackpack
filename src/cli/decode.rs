use std::fs;

if_tracing! {
    use voxell_timer::time_fn;
}

use crate::{
    cli::{DecodeArgs, pipeline},
    mutator::Mutator,
};

pub fn decode(args: DecodeArgs) {
    let input_path = &args.input;
    let output_path = &args.output;
    let mut pipeline = pipeline::build_pipeline(args.pipeline_selection());

    let compressed_data = fs::read(input_path).expect("Failed to read input file");
    let mut decompressed_data = Vec::new();
    if_tracing! {{
        let ((), decomp_dur) = time_fn(|| {
            pipeline
                .revert_mutation(&compressed_data, &mut decompressed_data)
                .expect("Decompression failed")
        });
        tracing::info!(event = "decode_complete", input = %input_path.display(), output = %output_path.display(), elapsed_ms = ?decomp_dur, decompressed_len = decompressed_data.len(), "decode finished");
    }};
    if_not_tracing! {{
        pipeline
            .revert_mutation(&compressed_data, &mut decompressed_data)
            .expect("Decompression failed");
    }};
    fs::write(output_path, decompressed_data).expect("Failed to write output file");
}
