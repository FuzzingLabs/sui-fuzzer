use clap::Parser;
use detector::detector::AvailableDetector;

use crate::fuzzer::config::Config;
use crate::fuzzer::fuzzer::Fuzzer;

mod detector;
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
    #[arg(long, required_unless_present = "list_functions")]
    target_module: Option<String>,

    /// The function to target
    #[arg(long, required_unless_present = "list_functions")]
    target_function: Option<String>,

    /// Detectors to use
    #[arg(
        short,
        long,
        value_delimiter = ',',
        required = false
    )]
    detectors: Option<Vec<AvailableDetector>>,

    /// Show list of functions starting with the prefix set in config
    #[arg(short, long)]
    list_functions: bool,
}

fn main() {
    let args = Args::parse();
    let config = Config::load_config(&args.config_path);
    if args.list_functions {
        println!(
            "Available functions starting with \"{}\":",
            config.fuzz_functions_prefix
        );
    } else if args.config_path != "" && args.target_function != None && args.target_module != None {
        let mut fuzzer = Fuzzer::new(
            config,
            &args.target_module.unwrap(),
            &args.target_function.unwrap(),
            args.detectors.as_ref(),
        );
        fuzzer.run();
    }
}
