use clap::ValueEnum;
use std::fmt::Debug;

use crate::fuzzer::{coverage::Coverage, error::Error};

use super::basic_opcode_detector::BasicOpCodeDetector;

/// Add new detectors here
#[derive(Debug, Clone, ValueEnum)]
pub enum AvailableDetector {
    All,
    BasicOpCodeDetector,
}

pub trait Detector {

    fn get_type(&self) -> AvailableDetector;
    fn detect(&self, coverage: &Coverage, error: Option<Error>) -> (bool, Option<String>);
}

/// Add new detectors here too
pub fn new_detector(d: &AvailableDetector) -> Box<dyn Detector> {
    match d {
        AvailableDetector::All => todo!(),
        AvailableDetector::BasicOpCodeDetector => Box::new(BasicOpCodeDetector::default()),
    }
}