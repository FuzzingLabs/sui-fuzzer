use bichannel::Channel;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::detector::detector::{new_detector, AvailableDetector, Detector};
use crate::fuzzer::coverage::Coverage;
use crate::fuzzer::error::Error;
use crate::fuzzer::stats::Stats;
use crate::mutator::mutator::Mutator;
use crate::mutator::rng::Rng;
use crate::mutator::types::Type;
use crate::runner::runner::Runner;

pub enum WorkerEvent {
    NewCrash(Vec<Type>, Error),
    CoverageUpdateRequest(HashSet<Coverage>),
    CoverageUpdateResponse(HashSet<Coverage>),
    DetectorTriggered(AvailableDetector, Option<String>)
}

pub struct Worker {
    channel: Channel<WorkerEvent, WorkerEvent>,
    stats: Arc<RwLock<Stats>>,
    runner: Box<dyn Runner>,
    mutator: Box<dyn Mutator>,
    coverage_set: HashSet<Coverage>,
    rng: Rng,
    execs_before_cov_update: u64,
    detectors: Vec<Box<dyn Detector>>,
}

impl Worker {
    pub fn new(
        channel: Channel<WorkerEvent, WorkerEvent>,
        stats: Arc<RwLock<Stats>>,
        runner: Box<dyn Runner>,
        mutator: Box<dyn Mutator>,
        seed: u64,
        execs_before_cov_update: u64,
        available_detectors: Option<Vec<AvailableDetector>>,
    ) -> Self {
        let mut detectors = vec![];
        if let Some(values) = available_detectors {
            detectors = Self::init_detectors(&values);
        }
        Worker {
            channel,
            stats,
            runner,
            mutator,
            coverage_set: HashSet::new(),
            rng: Rng {
                seed,
                exp_disabled: false,
            },
            execs_before_cov_update,
            detectors,
        }
    }

    fn init_detectors(detectors: &Vec<AvailableDetector>) -> Vec<Box<dyn Detector>> {
        let mut result = vec![];
        for detector in detectors {
            result.push(new_detector(detector));
        }
        result
    }

    fn pick_and_mutate_inputs(&mut self) -> Vec<Type> {
        let cov = self
            .coverage_set
            .iter()
            .nth(self.rng.rand(0, self.coverage_set.len() - 1))
            .unwrap();
        self.mutator.mutate(&cov.inputs, 4)
    }

    fn init_inputs(inputs: Vec<Type>) -> Vec<Type> {
        let mut res = vec![];
        for param in inputs {
            res.push(match param {
                Type::U8(_) => Type::U8(0),
                Type::U16(_) => Type::U16(0),
                Type::U32(_) => Type::U32(0),
                Type::U64(_) => Type::U64(0),
                Type::U128(_) => Type::U128(0),
                Type::Bool(_) => Type::Bool(true),
                Type::Vector(t, vec) => Type::Vector(t, Self::init_inputs(vec)),
            })
        }
        res
    }

    fn execute_detectors(&self, cov: &Coverage, err: Option<&Error>) {
        for detector in &self.detectors {
            let (detected, message) = detector.detect(cov, err.cloned());
            if detected {
                self.channel
                    .send(WorkerEvent::DetectorTriggered(detector.get_type(), message))
                    .unwrap();
            }
        }
    }

    pub fn run(&mut self) {
        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();
        let mut sec_elapsed = 0;

        // Input initialization
        let mut inputs = Self::init_inputs(self.runner.get_target_parameters());

        loop {
            let exec_result = self.runner.execute(inputs.clone());

            self.stats.write().unwrap().execs += 1;

            // Calculate execs_per_sec
            if execs_per_sec_timer.elapsed().as_secs() >= 1 {
                execs_per_sec_timer = Instant::now();
                sec_elapsed += 1;
                let tmp = self.stats.read().unwrap().execs;
                self.stats.write().unwrap().secs_since_last_cov += 1;
                self.stats.write().unwrap().execs_per_sec = tmp / sec_elapsed;
            }

            match exec_result {
                Ok(cov) => {
                    if let Some(coverage) = cov {
                        // Execute all activated detectors
                        self.execute_detectors(&coverage, None);
                        if !self.coverage_set.contains(&coverage) {
                            self.coverage_set.insert(coverage);
                            self.stats.write().unwrap().secs_since_last_cov = 0;
                        }
                    }
                }
                Err((coverage, error)) => {
                    // Execute all activated detectors
                    self.execute_detectors(&coverage, Some(&error));
                    if !self.coverage_set.contains(&coverage) {
                        self.coverage_set.insert(coverage);
                        self.stats.write().unwrap().secs_since_last_cov = 0;
                        self.channel
                            .send(WorkerEvent::NewCrash(inputs.clone(), error))
                            .unwrap();
                    }
                    self.stats.write().unwrap().crashes += 1;
                }
            }

            // Handle coverage updates every execs_before_cov_update execs (configurable in
            // configfile)
            if self.stats.read().unwrap().execs % self.execs_before_cov_update == 0 {
                self.channel
                    .send(WorkerEvent::CoverageUpdateRequest(
                        self.coverage_set.clone(),
                    ))
                    .unwrap();
            }

            // Check channel form main thread respone
            if let Ok(response) = self.channel.try_recv() {
                if let WorkerEvent::CoverageUpdateResponse(coverage_set) = response {
                    self.coverage_set.extend(coverage_set);
                }
            }

            // Updates input
            if self.coverage_set.len() > 0 {
                inputs = self.pick_and_mutate_inputs();
            }
        }
    }
}
