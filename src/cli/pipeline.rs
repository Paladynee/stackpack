use crate::{
    algorithms::{
        arcode::ArithmeticCoding,
        bwt::Bwt,
        mtf::Mtf,
        pipeline::{CompressionPipeline, get_specific_compressor_from_name},
    },
    cli::{PipelineCommand, PipelineSelection},
};

pub fn default_pipeline() -> CompressionPipeline {
    if_tracing! {
        tracing::info!(event = "using_default_pipeline", "using default compression pipeline");
    };
    CompressionPipeline::new()
        .with_algorithm(Bwt)
        .with_algorithm(Mtf)
        .with_algorithm(ArithmeticCoding)
}

pub fn get_preset(s: &str) -> Option<fn() -> CompressionPipeline> {
    Some(match s {
        "default" => default_pipeline,
        _ => None?,
    })
}

pub fn build_pipeline(selection: PipelineSelection) -> CompressionPipeline {
    match selection {
        PipelineSelection::Inline(string) => {
            let parts = string.split("->").map(|s| s.trim()).collect::<Vec<_>>();

            let mut pipeline = CompressionPipeline::new();

            for part in parts {
                if let Some(comp) = get_specific_compressor_from_name(part) {
                    pipeline.push_algorithm(comp);
                } else {
                    if_tracing! {
                        tracing::error!(event = "unknown_algorithm", algorithm = %part, "unknown algorithm specified in inline pipeline, skipping");
                    }
                    panic!()
                }
            }

            todo!()
        }
        PipelineSelection::FromFile(_path) => {
            todo!()
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
