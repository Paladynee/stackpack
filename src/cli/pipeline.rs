use std::fs;

use crate::{
    algorithms::pipeline::{CompressionPipeline, default_pipeline, get_preset, get_specific_compressor_from_name},
    cli::{PipelineCommand, PipelineSelection},
    plugins::LOADED_PLUGINS,
    registered::ALL_COMPRESSORS,
};

pub fn build_pipeline(selection: PipelineSelection) -> CompressionPipeline {
    match selection {
        PipelineSelection::Inline(string) => {
            let parts = string.split("->").map(|s| s.trim()).collect::<Vec<_>>();

            let mut pipeline = CompressionPipeline::new();

            for part in parts {
                if let Some(comp) = get_specific_compressor_from_name(part) {
                    pipeline.push_algorithm(comp.clone());
                } else {
                    if_tracing! {{
                        tracing::error!(event = "unknown_algorithm", algorithm = %part, "unknown algorithm specified in inline pipeline");
                    }}
                    panic!(
                        "unknown algorithm {:?}. you may have forgotten to enable plugins (unsafe), or not have the required plugins installed.",
                        part
                    );
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

pub fn pipeline(args: PipelineCommand) {
    match args {
        PipelineCommand::ListCompressors { detailed } => {
            for algo in ALL_COMPRESSORS.lock().iter() {
                if detailed && let Some(desc) = algo.short_description {
                    println!("Name: {}\nDescription: {}\n", algo.name, desc);
                } else {
                    println!("{}", algo.name);
                }
            }
        }
        PipelineCommand::ListPlugins => {
            let lock = LOADED_PLUGINS.lock();
            for item in lock.iter() {
                println!(
                    "Plugin loaded from: {:?}\nName: {}{}\n",
                    item.loaded_from,
                    item.api.short_name,
                    if let Some(desc) = item.api.description.as_option() {
                        format!("\nDescription: {}", desc)
                    } else {
                        String::new()
                    }
                );
            }
        }
        _ => todo!(),
    }
}
