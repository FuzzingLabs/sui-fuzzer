use clap::Parser;
use detector::detector::AvailableDetector;
use fuzzer::fuzzer_utils::replay;

use crate::fuzzer::config::Config;
use crate::fuzzer::fuzzer::Fuzzer;
use crate::runner::stateless_runner::sui_runner_utils::get_fuzz_functions_from_bin;

mod detector;
mod fuzzer;
mod mutator;
mod runner;
mod ui;
mod worker;

/// The *default* behavior of the fuzzer is to run in stateless mode

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the package to fuzz
    #[arg(short, long)]
    config_path: String,

    /// The function to target
    #[arg(long, required_unless_present_any = ["list_functions", "replay"])]
    target_module: Option<String>,

    /// The function to target
    #[arg(long, required_unless_present_any = ["list_functions", "replay"])]
    target_function: Option<String>,

    /// Detectors to use
    #[arg(short, long, value_delimiter = ',', required = false)]
    detectors: Option<Vec<AvailableDetector>>,

    /// Show list of functions starting with the prefix set in config
    #[arg(short, long)]
    list_functions: bool,

    /// Replay the given crash
    #[arg(short, long)]
    replay: Option<String>,

    /// Functions to fuzz statefully
    #[arg(short, long)]
    functions: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();
    let config = Config::load_config(&args.config_path);
    if args.list_functions {
        if let Some(target_module) = args.target_module {
            if let Some(contract_file) = config.contract {
                println!(
                    "Available functions starting with \"{}\":",
                    config.fuzz_functions_prefix
                );
                for function in get_fuzz_functions_from_bin(
                    &contract_file,
                    &target_module,
                    &config.fuzz_functions_prefix,
                ) {
                    println!("- {}", function);
                }
            } else {
                println!("Missing contract file in configuration !");
            }
        } else {
            println!("Missing target module !");
        }
    } else if args.config_path != "" {
        if let Some(target_module) = args.target_module {
            if let Some(target_function) = args.target_function {
                if let Some(_functions) = args.functions {
                    // Stateful
                    let mut fuzzer = Fuzzer::new(
                        config,
                        &target_module,
                        &target_function,
                        args.detectors.as_ref(),
                        true
                    );
                    fuzzer.run();
                } else {
                    // Stateless
                    let mut fuzzer = Fuzzer::new(
                        config,
                        &target_module,
                        &target_function,
                        args.detectors.as_ref(),
                        false
                    );
                    fuzzer.run();
                }
            }
        } else {
            if let Some(crashfile_path) = args.replay {
                replay(&config, &crashfile_path);
            }
        }
    }
}
