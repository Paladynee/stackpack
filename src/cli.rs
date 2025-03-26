//! cli component of the stackpack project.
//!
//! # Compression
//!
//! let's define some base cases for the cli invocations of this project to base the implementation on.
//! lines starting with `> ` denote commands that should be valid under the current implementation.
//! `$exename` stands for the executable name, which is `stackpack` in linux or `stackpack.exe` on windows, or
//! whatever the user renamed the file to. `<description>` denotes a required argument, while
//! `[description]` denotes an optional argument.
//!
//! > `$exename enc <path to file or folder> <output path>
//! >   [--using <pipeline name>]
//! >   [--from_file <path to pipeline file>]
//! >   [--embed_to_file]
//! >   [--preset <preset id>]
//! >   [--raw]`
//!
//! the first option passes the pipeline as a cli flag with custom parsing. this comes with two caveats:
//!     1. the decompressor must either remember the pipeline or manually store it elsewhere
//!     2. the decompressor has no way of inferring the pipeline used to encode the file, and thus cannot decompress it
//! > `$exename enc <path> <output path> --using "bwt -> rle -> arcode"`
//!
//! the next option reads the pipeline from a file, which allows the user to remember what pipeline was used to compress a file
//! in a readable format, allows for fine-grained experimenting with the pipeline for optimal compression,
//! and allows for sharing the pipeline with other users.
//! > `$exename enc <path> <output path> --from_file pipeline.json`
//!
//! another option is to use preset pipelines which can be invoked with a much shorter command.
//! > `$exename enc <path> <output path> --preset o1`
//!
//! another option is to use a default pipeline, which will be stabilized at some point and used if no other options are provided.
//! > `$exename enc <path> <output path>`
//!
//! now that the pipeline is determined, it is necessary to specify a method of storing (or not storing) the pipeline used to compress the input.
//! the first option uses a file format dedicated for this use case and embeds the pipeline information in the file itself.
//! this makes it much easier for the decompressor to decompress the resulting file, at the expense of being unreadable by the user,
//! and not allowing the raw bytes of the file to be passed into the decompressor.
//! the second option does not store this information at all, which allows the user to use the raw bytes of the file as they see fit
//! and to use the file with other programs, but the decompressor will be unable to decompress the file if the pipeline used to compress it is not remembered.
//! > `$exename enc <path> <output path> --raw`
//!
//! the third option outputs a `{file stem}.pipeline.json` file along with the compressed file,
//! which contains the pipeline in json format.
//!
//! > `$exename dec <path to file or folder> <output path>
//! >   [--using <pipeline name>]
//! >   [--from_file <path to pipeline file>]
//! >   [--preset <preset id>]
//! >   [--try-brute <depth>]`
//!
//! another option is to have a compressor repository. this repository has a `stackpack-config.json` file
//! that allows the decompressor to look up the pipeline used to compress the file based on the directory the file is in.
//! this solution is somewhat overkill and may be difficult to implement.
//!
//! now that the pipeline is determined and the information for all inputs and outputs is available, the pipeline is executed,
//! the bytes are encoded, and the file is wrapped in the specified format (if applicable) and stored. the program then terminates.
//!
//! # Decompression
//!
//! > `$exename dec <path to file> <output path> [--from_file <path to pipeline file>]`
//!
//! the decompressor first needs to know which pipeline was used to compress the file. there are four cases:
//!     1. the pipeline is stored in the file using the designated format (default case).
//!     2. the pipeline is specified as a cli argument, which will be used.
//!     3. the pipeline is stored in a file, which will be used.
//!     4. the pipeline is not known, and the decompressor will fail to decompress the file.
//!
//! for the first case, the file format is parsed and the pipeline is extracted.
//! for the second case, the pipeline is parsed from the cli argument as a string.
//! for the third case, the pipeline is read from the file in json format.
//! for the fourth case, if the `--try-brute N` flag is specified, the `format_validity_check` method of every available compressor is used
//! to recursively attempt decompression of the file up to the specified depth. the depth must be specified because some compressors do not fail for any input,
//! potentially causing infinite decompression. this should be a last resort option and avoided if possible.
//!
//! # Testing
//!
//! > `$exename test <path to file or folder> <output path>
//! >   [--using <pipeline name>]
//! >   [--from_file <path to pipeline file>]
//! >   [--preset <preset id>]`
//!
//! the testing mode is used to verify that the pipelines produce the same output as the original file. the pipeline input is handled in the same manner as in the other modes.
//! the program compresses the file using the pipeline, then immediately decompresses the output and compares the original file with the roundtripped file.
//! if a discrepancy is found, the compressed and decompressed data are written to the output path.
//!
//! # Pipeline Management
//!
//! > `$exename pipeline <subcommand> [args]`
//!
//! the pipeline mode is provided for viewing and managing pipelines, compressors, and their versions. currently there are two modes:
//!     1. list-compressors
//!     2. save-to-file
//!
//! > `$exename pipeline list-compressors [--detailed]`
//!
//! this command lists all compression algorithms available in the current build, along with their versions.
//! if the `--detailed` flag is passed, a description of what each algorithm is used for, its optimal usage scenarios,
//! and a short description of its internals is printed.
//!
//! > `$exename pipeline save-to-file <pipeline string> <output path>`
//!
//! this command converts a pipeline string into json format and saves it to the specified file.
//! the pipeline string is of the form:
//!     "pipeline_name1 -> pipeline_name2 -> ... -> pipeline_nameN"
//! the order of pipelines is specified in encoding order, meaning that when encoding, "pipeline_name1" is applied first,
//! followed by "pipeline_name2", and so on.
#![allow(unused)] //todo
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

use crate::algorithms;
use crate::algorithms::pipeline::CompressionPipeline;
use crate::compressor::Compressor;
use crate::compressor::CompressorExt;

/// Error types for CLI operations
#[derive(Debug, Error)]
pub enum CliError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Invalid pipeline format: {0}")]
    PipelineFormat(String),

    #[error("Unknown preset: {0}")]
    UnknownPreset(String),

    #[error("Unknown algorithm: {0}")]
    UnknownAlgorithm(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, CliError>;

/// CLI arguments for the stackpack application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Supported commands for stackpack
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Encode (compress) a file or folder
    #[command(alias = "enc")]
    Encode(EncodeArgs),

    /// Decode (decompress) a file or folder
    #[command(alias = "dec")]
    Decode(DecodeArgs),

    /// Test compression/decompression roundtrip
    Test(TestArgs),

    /// Pipeline management commands
    Pipeline {
        #[command(subcommand)]
        command: PipelineCommand,
    },
}

/// Arguments specific to the encode command
#[derive(Args, Debug)]
pub struct EncodeArgs {
    /// Path to the input file or folder
    pub input_path: PathBuf,

    /// Path for the output file or folder
    pub output_path: PathBuf,

    /// Specify pipeline as a string (e.g. "bwt -> mtf -> rle -> arcode")
    #[arg(long)]
    pub using: Option<String>,

    /// Load pipeline from a JSON file
    #[arg(long)]
    pub from_file: Option<PathBuf>,

    /// Embed pipeline information in the output file
    #[arg(long)]
    pub embed_to_file: bool,

    /// Use a predefined pipeline preset
    #[arg(long)]
    pub preset: Option<String>,

    /// Output raw compressed data without additional metadata
    #[arg(long)]
    pub raw: bool,
}

/// Arguments specific to the decode command
#[derive(Args, Debug)]
pub struct DecodeArgs {
    /// Path to the input file or folder
    pub input_path: PathBuf,

    /// Path for the output file or folder
    pub output_path: PathBuf,

    /// Specify pipeline as a string (e.g. "bwt -> mtf -> rle -> arcode")
    #[arg(long)]
    pub using: Option<String>,

    /// Load pipeline from a JSON file
    #[arg(long)]
    pub from_file: Option<PathBuf>,

    /// Use a predefined pipeline preset
    #[arg(long)]
    pub preset: Option<String>,

    /// Try brute force pipeline detection up to specified depth
    #[arg(long)]
    pub try_brute: Option<u32>,
}

/// Arguments specific to the test command
#[derive(Args, Debug)]
pub struct TestArgs {
    /// Path to the original file or folder
    pub input_path: PathBuf,

    /// Path for the roundtrip output file or folder
    pub output_path: PathBuf,

    /// Specify pipeline as a string (e.g. "bwt -> mtf -> rle -> arcode")
    #[arg(long)]
    pub using: Option<String>,

    /// Load pipeline from a JSON file
    #[arg(long)]
    pub from_file: Option<PathBuf>,

    /// Use a predefined pipeline preset
    #[arg(long)]
    pub preset: Option<String>,
}

/// Pipeline management subcommands
#[derive(Subcommand, Debug)]
pub enum PipelineCommand {
    /// List available compression algorithms
    ListCompressors {
        /// Show detailed information about each compressor
        #[arg(long)]
        detailed: bool,
    },

    /// Save a pipeline configuration to a file
    SaveToFile {
        /// Pipeline string (e.g. "bwt -> mtf -> rle -> arcode")
        pipeline: String,

        /// Output file path
        output_path: PathBuf,
    },
}

/// Serializable pipeline configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PipelineConfig {
    /// Names of algorithms in the pipeline
    pub algorithms: Vec<String>,

    /// Optional algorithm-specific parameters
    #[serde(default)]
    pub parameters: Vec<serde_json::Value>,

    /// Version information
    pub version: String,
}

/// Pipeline preset configurations
pub struct PipelinePresets;

impl PipelinePresets {
    /// Get a pipeline configuration from a preset name
    pub fn get_preset(name: &str) -> Result<PipelineConfig> {
        match name {
            "o1" => Ok(PipelineConfig {
                algorithms: vec!["bwt".to_string(), "mtf".to_string(), "rle".to_string(), "arcode".to_string()],
                parameters: Vec::new(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            "fast" => Ok(PipelineConfig {
                algorithms: vec!["rle".to_string(), "arcode".to_string()],
                parameters: Vec::new(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            "text" => Ok(PipelineConfig {
                algorithms: vec!["bwt".to_string(), "mtf".to_string(), "arcode".to_string()],
                parameters: Vec::new(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            _ => Err(CliError::UnknownPreset(name.to_string())),
        }
    }
}

/// Pipeline string parser
pub struct PipelineParser;

impl PipelineParser {
    /// Parse a pipeline string into a PipelineConfig
    pub fn parse(pipeline_str: &str) -> Result<PipelineConfig> {
        let parts: Vec<&str> = pipeline_str.split("->").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return Err(CliError::PipelineFormat("Empty pipeline string".to_string()));
        }

        Ok(PipelineConfig {
            algorithms: parts.iter().map(|s| s.to_string()).collect(),
            parameters: Vec::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

/// Algorithm parser
pub struct AlgorithmParser;

impl AlgorithmParser {
    /// Parse a single algorithm from its short name or long name.
    pub fn parse(name: &str) -> Result<Box<dyn CompressorExt>> {
        for algo in algorithms::all() {
            for alias in algo.aliases() {
                if &name == alias {
                    return Ok(algo.dyn_clone());
                }
            }
        }

        Err(CliError::UnknownAlgorithm(name.to_string()))
    }
}

/// Pipeline builder
pub struct PipelineBuilder;

impl PipelineBuilder {
    /// Build a pipeline from a PipelineConfig
    pub fn build_pipeline(config: &PipelineConfig) -> Result<CompressionPipeline> {
        let mut pipeline = CompressionPipeline::new();

        for algo_name in &config.algorithms {
            match algo_name.as_str() {
                "lol" => {}
                // TODO: Implement algorithm factory based on name
                // This should create and add appropriate algorithms to the pipeline
                _ => return Err(CliError::UnknownAlgorithm(algo_name.clone())),
            }
        }

        Ok(pipeline)
    }

    /// Build a pipeline from various sources with priority:
    /// 1. Direct pipeline string (--using)
    /// 2. Pipeline file (--from_file)
    /// 3. Preset (--preset)
    /// 4. Default pipeline
    pub fn build_from_args(using: Option<&str>, from_file: Option<&PathBuf>, preset: Option<&str>) -> Result<CompressionPipeline> {
        // TODO: Implement pipeline building logic based on input priority
        todo!("Implement pipeline building from arguments")
    }
}

/// File format handler for compression output
pub struct FileFormatHandler;

impl FileFormatHandler {
    /// Wrap compressed data with metadata if needed
    pub fn wrap_compressed_data(data: Vec<u8>, config: &PipelineConfig, embed_to_file: bool) -> Result<Vec<u8>> {
        if embed_to_file {
            // TODO: Implement embedding pipeline metadata into the file
            todo!("Implement metadata embedding")
        } else {
            Ok(data)
        }
    }

    /// Extract pipeline configuration from compressed file
    pub fn extract_pipeline_config(data: &[u8]) -> Result<Option<PipelineConfig>> {
        // TODO: Implement pipeline extraction from file metadata
        todo!("Implement pipeline extraction from file")
    }

    /// Save pipeline configuration to a separate file
    pub fn save_pipeline_config(config: &PipelineConfig, base_path: &PathBuf) -> Result<()> {
        let mut config_path = base_path.clone();
        let file_name = config_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| CliError::Io(io::Error::new(io::ErrorKind::InvalidInput, "Invalid output path")))?;

        config_path.set_file_name(format!("{}.pipeline.json", file_name));

        let json = serde_json::to_string_pretty(config)?;
        fs::write(config_path, json)?;

        Ok(())
    }
}

/// Command execution functions
pub fn execute_command(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Encode(args) => execute_encode(args),
        Command::Decode(args) => execute_decode(args),
        Command::Test(args) => execute_test(args),
        Command::Pipeline { command } => execute_pipeline_command(command),
    }
}

fn execute_encode(args: EncodeArgs) -> Result<()> {
    // TODO: Implement encode command execution
    todo!("Implement encode command")
}

fn execute_decode(args: DecodeArgs) -> Result<()> {
    // TODO: Implement decode command execution
    todo!("Implement decode command")
}

fn execute_test(args: TestArgs) -> Result<()> {
    // TODO: Implement test command execution
    todo!("Implement test command")
}

fn execute_pipeline_command(cmd: PipelineCommand) -> Result<()> {
    match cmd {
        PipelineCommand::ListCompressors { detailed } => {
            // TODO: Implement listing compressors
            todo!("Implement list-compressors command")
        }
        PipelineCommand::SaveToFile { pipeline, output_path } => {
            let config = PipelineParser::parse(&pipeline)?;
            let json = serde_json::to_string_pretty(&config)?;
            fs::write(output_path, json)?;
            Ok(())
        }
    }
}

/// Function to parse CLI arguments and execute appropriate command
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    execute_command(cli)
}
