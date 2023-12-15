use std::{time::SystemTime, fs::{File, self}, io::Write};

use chrono::{DateTime, Utc};

use super::crash::Crash;

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