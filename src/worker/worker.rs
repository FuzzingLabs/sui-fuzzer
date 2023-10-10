use bichannel::Channel;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;
use std::time::Instant;

use crate::fuzzer::stats::Stats; use crate::runner::runner::Runner;
use crate::mutator::mutator::Mutator;
use crate::mutator::rng::Rng;
use crate::fuzzer::coverage::Coverage;

pub enum WorkerEvent {
    NewCrash(Vec<u8>, String),
    CoverageUpdateRequest(HashSet<Coverage>),
    CoverageUpdateResponse(HashSet<Coverage>)
}

pub struct Worker {
    channel: Channel<WorkerEvent, WorkerEvent>,
    stats: Arc<RwLock<Stats>>,
    runner: Box<dyn Runner<InputType = Vec<u8>>>,
    mutator: Box<dyn Mutator>,
    coverage_set: HashSet<Coverage>,
    rng: Rng,
    execs_before_cov_update: u64
}

impl Worker {

    pub fn new
        (
            channel: Channel<WorkerEvent, WorkerEvent>, 
            stats: Arc<RwLock<Stats>>,
            runner: Box<dyn Runner<InputType = Vec<u8>>>,
            mutator: Box<dyn Mutator>,
            seed: u64,
            execs_before_cov_update: u64
        ) -> Self {
            Worker {
                channel,
                stats,
                runner,
                mutator,
                coverage_set: HashSet::new(),
                rng: Rng { seed, exp_disabled: false },
                execs_before_cov_update
            }
        }

    fn pick_and_mutate_input(&mut self) -> Vec<u8> {
        let cov = self.coverage_set.iter().nth(self.rng.rand(0, self.coverage_set.len() - 1)).unwrap();
        self.mutator.mutate(&cov.input, 4)
    }

    pub fn run(&mut self) {

        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();
        let mut sec_elapsed = 0;

        // Input initialization
        let mut input = vec![0x61u8, 0x61u8, 0x61u8, 0x61u8, 0x61u8, 0x61u8];
        input = self.mutator.mutate(&input, 4);

        loop {

            let exec_result = self.runner.execute(input.clone());

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
                        if !self.coverage_set.contains(&coverage) {
                            self.coverage_set.insert(coverage);
                            self.stats.write().unwrap().secs_since_last_cov = 0;
                        }
                    }
                },
                Err((coverage, msg)) => {
                    if !self.coverage_set.contains(&coverage) {
                        self.coverage_set.insert(coverage);
                        self.stats.write().unwrap().secs_since_last_cov = 0;
                        self.channel.send(WorkerEvent::NewCrash(input.clone(), msg)).unwrap();
                    }
                    self.stats.write().unwrap().crashes += 1;
                }
            }

            // Handle coverage updates every execs_before_cov_update execs (configurable in
            // configfile)
            if self.stats.read().unwrap().execs % self.execs_before_cov_update == 0 {
                self.channel.send(WorkerEvent::CoverageUpdateRequest(self.coverage_set.clone())).unwrap();
            }

            // Check channel form main thread respone
            if let Ok(response) = self.channel.try_recv() {
                if let WorkerEvent::CoverageUpdateResponse(coverage_set) = response {
                    self.coverage_set.extend(coverage_set);
                }
            }

            // Updates input
            if self.coverage_set.len() > 0 {
                input = self.pick_and_mutate_input();
            }
        }
    }
}
