use bichannel::Channel;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;
use std::time::Instant;

use crate::fuzzer::stats::Stats;
use crate::runner::runner::Runner;
use crate::mutator::mutator::Mutator;
use crate::mutator::rng::Rng;
use crate::fuzzer::coverage::Coverage;

pub enum WorkerEvent {
    NewCoverage(Vec<u8>),
}

pub struct Worker {
    channel: Channel<WorkerEvent, u8>,
    stats: Arc<RwLock<Stats>>,
    runner: Box<dyn Runner<InputType = Vec<u8>>>,
    mutator: Box<dyn Mutator>,
    coverage_set: HashSet<Coverage>,
    rng: Rng
}

impl Worker {

    pub fn new(channel: Channel<WorkerEvent, u8>, 
               stats: Arc<RwLock<Stats>>,
               runner: Box<dyn Runner<InputType = Vec<u8>>>,
               mutator: Box<dyn Mutator>,
               seed: u64) -> Self {
        Worker {
            channel,
            stats,
            runner,
            mutator,
            coverage_set: HashSet::new(),
            rng: Rng { seed, exp_disabled: false }
        }
    }

    fn pick_and_mutate_input(&mut self) -> Vec<u8> {
        let cov = self.coverage_set.iter().nth(self.rng.rand(0, self.coverage_set.len() - 1)).unwrap();
        self.mutator.mutate(&cov.input, 8)
    }

    pub fn run(&mut self) {

        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();
        let mut sec_elapsed = 0;

        let mut input = vec![0x61u8, 0x61u8, 0x61u8, 0x61u8, 0x61u8, 0x61u8];

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
                        if !coverage.data.is_empty() && !self.coverage_set.contains(&coverage) {
                            self.stats.write().unwrap().secs_since_last_cov = 0;
                            self.coverage_set.insert(coverage);
                            self.channel.send(WorkerEvent::NewCoverage(input.clone())).unwrap();
                        }
                    }
                },
                Err((coverage, _msg)) => {
                    if !coverage.data.is_empty() && !self.coverage_set.contains(&coverage) {
                        self.coverage_set.insert(coverage);
                        self.channel.send(WorkerEvent::NewCoverage(input.clone())).unwrap();
                    }
                    // eprintln!("ERROR {}", msg);
                    self.stats.write().unwrap().crashes += 1;
                }
            }
            input = self.pick_and_mutate_input();
        }
    }
}
