use std::{time::SystemTime, fs::{File, self}, io::Write};

use chrono::{DateTime, Utc};

use crate::runner::{sui_runner::SuiRunner, runner::Runner};

use super::{crash::Crash, config::Config};

pub fn write_crashfile(path: &str, crash: Crash) {
    if let Err(err) = fs::create_dir_all(path) {
        panic!("Could not create crashes directory: {}", err);
    }
    let d = SystemTime::now();
    // Create DateTime from SystemTime
    let datetime = DateTime::<Utc>::from(d);
    // Formats the combined date and time with the specified format string.
    let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
    let mut file = File::create(format!("{}/{}-{}.json", path, timestamp_str, crash.target_function)).unwrap();
    file.write_all(serde_json::to_string(&crash).unwrap().as_bytes()).unwrap();
}

pub fn replay(config: &Config, crashfile_path: &str) {
    let data = fs::read_to_string(crashfile_path).expect("Could not read crash file !");
    let crash: Crash = serde_json::from_str(&data).expect("Could not load crash file !");

    if let Some(contract_file) = &config.contract_file {
        let mut runner = SuiRunner::new(&contract_file, &crash.target_module, &crash.target_function);
        match runner.execute(crash.inputs) {
            Ok(_) => unreachable!(),
            Err(e) => println!("{:?}", e.1),
        }
    }
}