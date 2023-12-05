use clap::Parser;

use crate::fuzzer::config::Config;
use crate::fuzzer::fuzzer::Fuzzer;

mod fuzzer;
mod mutator;
mod runner;
mod ui;
mod worker;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the package to fuzz
    #[arg(short, long)]
    config_path: String,

    /// The function to target
    #[arg(short, long)]
    target_module: String,

    /// The function to target
    #[arg(short, long)]
    target_function: String,
}

fn main() {
    let args = Args::parse();
    if args.config_path != "" && args.target_function != "" && args.target_module != "" {
        let config = Config::load_config(&args.config_path);
        let mut fuzzer = Fuzzer::new(config, &args.target_module, &args.target_function);
        fuzzer.run();
    }
}
