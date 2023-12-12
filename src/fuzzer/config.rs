use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Config {
    // Whether to use the ui or not
    pub use_ui: bool,
    // Number of threads (worker) started
    pub nb_threads: u8,
    // Rng seed
    pub seed: Option<u64>,
    // Initialization parameter of the runner (can be anything since it a string (json, base64, ...))
    pub contract_file: Option<String>,
    // How many execs before coverage update
    pub execs_before_cov_update: u64,
    // Where to put the corpus
    pub corpus_dir: String,
    // Where to put the crash files
    pub crashes_dir: String,
    // Fuzzing functions prefix
    pub fuzz_functions_prefix: String
}

impl Config {
    #[allow(dead_code)]
    pub fn default() -> Self {
        Config {
            use_ui: true,
            nb_threads: 8,
            seed: Some(4284),
            contract_file: None,
            execs_before_cov_update: 10_000,
            corpus_dir: "./corpus".to_string(),
            crashes_dir: "./crashes".to_string(),
            fuzz_functions_prefix: "fuzz_".to_string()
        }
    }

    #[allow(dead_code)]
    pub fn load_config(path: &str) -> Self {
        let config_string = fs::read_to_string(path).expect("Unable to read config file");
        return serde_json::from_str(&config_string).expect("Could not parse json config file");
    }
}
