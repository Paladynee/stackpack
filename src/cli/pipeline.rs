use std::fs;

use crate::{
    algorithms::pipeline::{CompressionPipeline, default_pipeline, get_preset, get_specific_compressor_from_name},
    cli::{PipelineCommand, PipelineSelection},
};

pub fn build_pipeline(selection: PipelineSelection) -> CompressionPipeline {
    match selection {
        PipelineSelection::Inline(string) => {
            let parts = string.split("->").map(|s| s.trim()).collect::<Vec<_>>();

            let mut pipeline = CompressionPipeline::new();

            for part in parts {
                if let Some(comp) = get_specific_compressor_from_name(part) {
                    pipeline.push_algorithm(comp.mutator);
                } else {
                    if_tracing! {
                        tracing::error!(event = "unknown_algorithm", algorithm = %part, "unknown algorithm specified in inline pipeline, skipping");
                    }
                    panic!("unknown_algorithm")
                }
            }

            pipeline
        }
        PipelineSelection::FromFile(path) => {
            let data = fs::read(&path).expect("couldn't read pipeline file");
            CompressionPipeline::try_from_bytes(&data).expect("pipeline file corrupt")
        }
        PipelineSelection::Preset(preset_name) => match get_preset(&preset_name) {
            Some(t) => t(),
            None => default_pipeline(),
        },
        PipelineSelection::Default => default_pipeline(),
    }
}

pub fn pipeline(_args: PipelineCommand) {
    todo!()
}
