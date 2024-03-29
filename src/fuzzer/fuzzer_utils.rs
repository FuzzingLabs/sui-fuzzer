use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    time::SystemTime,
};
use chrono::{DateTime, Utc};
use crate::runner::{runner::Runner, stateless_runner::sui_runner::SuiRunner};
use super::{config::Config, coverage::Coverage, crash::Crash};

pub fn write_crashfile(path: &str, crash: Crash) {
    if let Err(err) = fs::create_dir_all(path) {
        panic!("Could not create crashes directory: {}", err);
    }
    let d = SystemTime::now();
    // Create DateTime from SystemTime
    let datetime = DateTime::<Utc>::from(d);
    // Formats the combined date and time with the specified format string.
    let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
    let mut file = File::create(format!(
        "{}/{}-{}.json",
        path, timestamp_str, crash.target_function
    ))
    .unwrap();
    file.write_all(serde_json::to_string(&crash).unwrap().as_bytes())
        .unwrap();
}

pub fn write_corpusfile(path: &str, cov: &Coverage) {
    if let Err(err) = fs::create_dir_all(path) {
        panic!("Could not create crashes directory: {}", err);
    }
    let d = SystemTime::now();
    // Create DateTime from SystemTime
    let datetime = DateTime::<Utc>::from(d);
    // Formats the combined date and time with the specified format string.
    let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
    let mut file = File::create(format!("{}/{}.json", path, timestamp_str)).unwrap();
    file.write_all(serde_json::to_string(&cov).unwrap().as_bytes())
        .unwrap();
}

pub fn replay(config: &Config, crashfile_path: &str) {
    let data = fs::read_to_string(crashfile_path).expect("Could not read crash file !");
    let crash: Crash = serde_json::from_str(&data).expect("Could not load crash file !");

    if let Some(contract_file) = &config.contract {
        let mut runner =
            SuiRunner::new(&contract_file, &crash.target_module, &crash.target_function);
        match runner.execute(crash.inputs) {
            Ok(_) => unreachable!(),
            Err(e) => println!("{:?}", e.1),
        }
    }
}

pub fn load_corpus(path: &str) -> Result<HashSet<Coverage>, String> {
    let mut set = HashSet::new();
    if let Ok(paths) = fs::read_dir(path) {
        for file in paths {
            let data = fs::read_to_string(file.unwrap().path().display().to_string())
                .expect("Could not read corpus file !");
            let coverage = serde_json::from_str(&data).unwrap();
            set.insert(coverage);
        }
        return Ok(set);
    }
    Err("Could not read corpus directory !".to_string())
}

pub fn load_crashes(path: &str) -> Result<HashSet<Crash>, String> {
    let mut set = HashSet::new();
    if let Ok(paths) = fs::read_dir(path) {
        for file in paths {
            let data = fs::read_to_string(file.unwrap().path().display().to_string())
                .expect("Could not read crash file !");
            let crash = serde_json::from_str(&data).unwrap();
            set.insert(crash);
        }
        return Ok(set);
    }
    Err("Could not read crash directory !".to_string())
}
