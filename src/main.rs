#![allow(unused_labels)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

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

extern crate anyhow;
extern crate arcode;
extern crate clap;
// extern crate derive_fromstr;
extern crate libsais;
// extern crate lzw;
// extern crate log;
// extern crate no_panic;
// extern crate serde;
// extern crate serde_json;
// extern crate thiserror;
// extern crate voxell_rng;
extern crate voxell_timer;
extern crate walkdir;
if_tracing! {
    extern crate tracing;
    extern crate tracing_log;
    extern crate tracing_subscriber;
}

if_tracing! {
    use tracing_subscriber::fmt;
}

use crate::cli::{Cli, Command};
use clap::Parser;

mod algorithms;
mod cli;
mod mutator;

fn main() {
    if_tracing! {
        let max_level = {
            fn parse_level(s: &str) -> Option<tracing::Level> {
                let first = s.split(',').next()?.trim();
                let level_part = match first.find('=') {
                    Some(pos) => &first[pos + 1..],
                    None => first,
                }
                .trim()
                .to_ascii_lowercase();

                match level_part.as_str() {
                    "trace" => Some(tracing::Level::TRACE),
                    "debug" => Some(tracing::Level::DEBUG),
                    "info" => Some(tracing::Level::INFO),
                    "warn" | "warning" => Some(tracing::Level::WARN),
                    "error" => Some(tracing::Level::ERROR),
                    _ => None,
                }
            }
            std::env::var("RUST_LOG")
                .ok()
                .and_then(|s| parse_level(&s))
                .unwrap_or(tracing::Level::ERROR)
        };

        let subscriber = fmt()
            .with_max_level(max_level)
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
