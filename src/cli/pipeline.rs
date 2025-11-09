use crate::{
    algorithms::{arcode::ArithmeticCoding, bwt::Bwt, mtf::Mtf, pipeline::CompressionPipeline},
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
            if_tracing! {
                tracing::info!(event = "using_inline_pipeline", "using inline pipeline: {}", string);
            };
            default_pipeline()
        }
        PipelineSelection::FromFile(path) => {
            if_tracing! {
                tracing::info!(event = "using_file_pipeline", "using pipeline from file: {}", path.display());
            };
            default_pipeline()
        }
        PipelineSelection::Preset(preset_name) => match get_preset(&preset_name) {
            Some(t) => t(),
            None => default_pipeline(),
        },
        PipelineSelection::Default => default_pipeline(),
    }
}

pub fn pipeline(args: PipelineCommand) {}
