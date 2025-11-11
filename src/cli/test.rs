use crate::cli::{TestArgs, corpus::run_folder};

pub fn test(args: TestArgs) {
    run_folder(&args.input, args.pipeline_selection(), args.write_files_if_failed);
}
