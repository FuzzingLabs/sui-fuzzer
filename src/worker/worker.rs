use std::collections::HashSet;

use crate::{
    detector::detector::AvailableDetector,
    fuzzer::{coverage::Coverage, crash::Crash, error::Error},
    mutator::types::Type,
};

#[derive(Clone)]
pub enum WorkerEvent {
    NewCrash(String, Vec<Type>, Error),
    NewUniqueCrash(Crash),
    CoverageUpdateRequest(HashSet<Coverage>),
    CoverageUpdateResponse(HashSet<Coverage>),
    DetectorTriggered(AvailableDetector, Option<String>),
}

pub trait Worker{
    fn run(&mut self);
}
