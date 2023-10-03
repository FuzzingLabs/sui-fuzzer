use clap::Parser;

use crate::fuzzer::fuzzer::Fuzzer;
use crate::fuzzer::config::Config;

mod fuzzer;
mod runner;
mod worker;
mod ui;
mod mutator;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the package to fuzz
    #[arg(short, long)]
    module_path: String,
}

fn main() {
    let args = Args::parse();
    if args.module_path != "" {
        let config = Config {
            runner_parameter: Some(args.module_path),
            ..Config::default()
        };
        let mut fuzzer = Fuzzer::new(config);
        fuzzer.run();
    }
}
