#![allow(unused_labels)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
extern crate anyhow;
extern crate arcode;
extern crate clap;
extern crate derive_fromstr;
extern crate libsais;
extern crate lzw;
extern crate no_panic;
extern crate serde;
extern crate serde_json;
extern crate thiserror;
extern crate voxell_rng;
extern crate voxell_timer;
extern crate walkdir;

#[macro_export]
macro_rules! if_tracing {
    {$($body:tt)*} => {
        ::cfg_if::cfg_if! {
            if #[cfg(feature = "tracing")] {
                $($body)*
            }
        }
    };
}

if_tracing! {
    use tracing_subscriber::{EnvFilter, fmt};
}

use crate::cli::{Cli, Command};
use clap::Parser;

mod algorithms;
mod cli;
mod compressor;

fn main() {
    if_tracing! {
        let subscriber = fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(false)
            .finish();
        tracing::subscriber::set_global_default(subscriber).ok();
    }

    let cli = Cli::parse();
    match cli.command {
        Command::Encode(args) => cli::encode::encode(args),
        Command::Decode(args) => cli::decode::decode(args),
        Command::Test(args) => cli::test::test(args),
        Command::Corpus(args) => cli::corpus::corpus(args),
        Command::Pipeline(command) => cli::pipeline::pipeline(command),
    }
}
