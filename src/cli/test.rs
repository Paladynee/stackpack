use crate::cli::{TestArgs, pipeline};
use std::process;

pub fn test(args: TestArgs) {
    let mut _pipeline = pipeline::build_pipeline(args.pipeline_selection());
    eprintln!("the 'test' subcommand is not implemented yet");
    process::exit(1);
}
