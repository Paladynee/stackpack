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
//! > `$exename enc <path> <output path> --from_file pipeline.stp`
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
pub mod corpus;
pub mod decode;
pub mod encode;
pub mod pipeline;
pub mod test;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "stackpack",
    author,
    version,
    about = "A compressor-agnostic compression pipeline CLI.",
    long_about = None,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(long = "unsafe", global = true, help = "Enable things which can't be checked for safety (plugins)")]
    pub unsafe_mode: bool,
    #[command(subcommand)]
    pub command: Command,
}

/// Supported stackpack subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(name = "enc", aliases = ["e", "encode", "c", "compress", "a", "archive"], about = "Compress input data using a pipeline.")]
    Encode(EncodeArgs),
    #[command(name = "dec", aliases = ["d", "decode", "decompress", "u", "uncompress", "unpack"], about = "Decompress data produced by stackpack.")]
    Decode(DecodeArgs),
    #[command(name = "test", about = "Round-trip pipelines against input data.")]
    Test(TestArgs),
    #[command(name = "pipeline", about = "Inspect or manage available pipelines.", subcommand)]
    Pipeline(PipelineCommand),
    #[command(name = "corpus", about = "Run corpus compression benchmarks.")]
    Corpus(CorpusArgs),
}

/// Common selectors for pipeline inputs.
#[derive(Debug, Args, Clone, Default)]
pub struct PipelineSelector {
    #[arg(
		long = "using",
		value_name = "PIPELINE",
		conflicts_with_all = ["from_file", "preset"],
		help = "Inline pipeline description, e.g. \"bwt -> mtf -> arcode\"."
	)]
    pub inline: Option<String>,
    #[arg(
		long = "from_file",
		value_name = "PIPELINE_FILE",
		conflicts_with_all = ["inline", "preset"],
		help = "Path to a JSON pipeline definition file."
	)]
    pub from_file: Option<PathBuf>,
    #[arg(
		long = "preset",
		value_name = "PRESET",
		conflicts_with_all = ["inline", "from_file"],
		help = "Preset pipelines registered by stackpack."
	)]
    pub preset: Option<String>,
}

impl PipelineSelector {
    /// Resolve to a concrete pipeline selection, defaulting when no option is provided.
    pub fn selection(&self) -> PipelineSelection {
        if let Some(inline) = &self.inline {
            PipelineSelection::Inline(inline.clone())
        } else if let Some(path) = &self.from_file {
            PipelineSelection::FromFile(path.clone())
        } else if let Some(preset) = &self.preset {
            PipelineSelection::Preset(preset.clone())
        } else {
            PipelineSelection::Default
        }
    }
}

/// Where the pipeline description should be sourced from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineSelection {
    Inline(String),
    FromFile(PathBuf),
    Preset(String),
    Default,
}

/// Packaged pipeline persistence strategy.
#[derive(Debug, Args, Clone, Copy, Default)]
pub struct PipelinePersistenceArgs {
    #[arg(
        long = "embed_to_file",
        conflicts_with = "raw",
        help = "Embed the pipeline metadata directly in the output artifact."
    )]
    pub embed_to_file: bool,
    #[arg(
        long,
        conflicts_with = "embed_to_file",
        help = "Do not store pipeline metadata alongside the compressed output."
    )]
    pub raw: bool,
}

impl PipelinePersistenceArgs {
    pub fn mode(&self) -> PipelinePersistence {
        if self.embed_to_file {
            PipelinePersistence::Embedded
        } else if self.raw {
            PipelinePersistence::Raw
        } else {
            PipelinePersistence::Sidecar
        }
    }
}

/// Encoding-time storage mode for pipeline metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelinePersistence {
    Sidecar,
    Embedded,
    Raw,
}

/// CLI arguments for the `enc` subcommand.
#[derive(Debug, Args, Clone)]
pub struct EncodeArgs {
    #[arg(value_name = "path/to/input", help = "Path to the file or directory to compress.")]
    pub input: PathBuf,
    #[arg(value_name = "path/to/output", help = "Destination path for the compressed output.")]
    pub output: PathBuf,
    #[command(flatten)]
    pub pipeline: PipelineSelector,
    #[command(flatten)]
    pub persistence: PipelinePersistenceArgs,
}

impl EncodeArgs {
    pub fn pipeline_selection(&self) -> PipelineSelection {
        self.pipeline.selection()
    }

    pub fn persistence_mode(&self) -> PipelinePersistence {
        self.persistence.mode()
    }
}

/// CLI arguments for the `dec` subcommand.
#[derive(Debug, Args, Clone)]
pub struct DecodeArgs {
    #[arg(value_name = "path/to/input", help = "Path to the file or directory to decompress.")]
    pub input: PathBuf,
    #[arg(value_name = "path/to/output", help = "Destination path for the decompressed data.")]
    pub output: PathBuf,
    #[command(flatten)]
    pub pipeline: PipelineSelector,
    #[arg(
		long = "try-brute",
		value_name = "depth",
		value_parser = parse_positive_depth,
		help = "Attempt brute-force decompression up to the provided pipeline depth."
	)]
    pub brute_force_depth: Option<usize>,
}

impl DecodeArgs {
    pub fn pipeline_selection(&self) -> PipelineSelection {
        self.pipeline.selection()
    }
}

/// CLI arguments for the `test` subcommand.
#[derive(Debug, Args, Clone)]
pub struct TestArgs {
    #[arg(value_name = "path/to/input", help = "Path to the file or directory to exercise.")]
    pub input: PathBuf,
    #[command(flatten)]
    pub pipeline: PipelineSelector,
    #[arg(
        long = "write_files_if_failed",
        help = "Write compressed and decompressed files to input directory if a test fails."
    )]
    pub write_files_if_failed: bool,
}

impl TestArgs {
    pub fn pipeline_selection(&self) -> PipelineSelection {
        self.pipeline.selection()
    }
}

/// CLI arguments for the `corpus` subcommand.
#[derive(Debug, Args, Clone)]
pub struct CorpusArgs {
    #[command(flatten)]
    pub pipeline: PipelineSelector,
}

impl CorpusArgs {
    pub fn pipeline_selection(&self) -> PipelineSelection {
        self.pipeline.selection()
    }
}

/// Pipeline inspection and management subcommands.
#[derive(Debug, Subcommand)]
pub enum PipelineCommand {
    #[command(name = "list-compressors", about = "List available compressors.")]
    ListCompressors {
        #[arg(long, help = "Print additional metadata for each compressor.")]
        detailed: bool,
    },
    #[command(name = "list-plugins", about = "List available plugins.")]
    ListPlugins,
    #[command(name = "save-to-file", about = "Persist a pipeline string to a file.")]
    SaveToFile {
        #[arg(value_name = "PIPELINE", help = "Pipeline string in \"a -> b -> c\" form.")]
        pipeline: String,
        #[arg(value_name = "path/to/output", help = "Output path for the pipeline file.")]
        output: PathBuf,
    },
}

fn parse_positive_depth(raw: &str) -> Result<usize, String> {
    let depth: usize = raw.parse().map_err(|err| format!("failed to parse depth '{raw}': {err}"))?;
    if depth == 0 {
        Err("depth must be greater than zero".to_string())
    } else {
        Ok(depth)
    }
}

pub fn warn_unsafe_mode_enabled() {
    eprintln!("[warn] stackpack: unsafe mode enabled, safety is not guaranteed.");
}
