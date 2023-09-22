use crate::fuzzer::fuzzer::Fuzzer;
use crate::fuzzer::config::Config;

mod fuzzer;
mod runner;
mod worker;
mod ui;
mod mutator;

fn main() {
    let config = Config {
        runner_parameter: Some(String::from("fuzzinglabs_package")),
        ..Config::default()
    };
    let mut fuzzer = Fuzzer::new(config);
    fuzzer.run();
}
