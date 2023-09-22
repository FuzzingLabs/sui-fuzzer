use bichannel::Channel;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;
use std::time::Instant;

use crate::fuzzer::stats::Stats;
use crate::runner::runner::Runner;
use crate::fuzzer::coverage::Coverage;

pub enum WorkerEvent {
    NewCoverage,
}

pub struct Worker {
    channel: Channel<WorkerEvent, u8>,
    stats: Arc<RwLock<Stats>>,
    runner: Box<dyn Runner>,
    coverage_set: HashSet<Vec<Coverage>>
}

impl Worker {

    pub fn new(channel: Channel<WorkerEvent, u8>, stats: Arc<RwLock<Stats>>, runner: Box<dyn Runner>) -> Self {
        Worker {
            channel,
            stats,
            runner,
            coverage_set: HashSet::new()
        }
    }

    pub fn run(&mut self) {

        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();
        let mut sec_elapsed = 0;

        loop {
            let exec_result = self.runner.execute();

            self.stats.write().unwrap().execs += 1;

            // Calculate execs_per_sec
            if execs_per_sec_timer.elapsed().as_secs() >= 1 {
                execs_per_sec_timer = Instant::now();
                sec_elapsed += 1;
                let tmp = self.stats.read().unwrap().execs;
                self.stats.write().unwrap().execs_per_sec = tmp / sec_elapsed;
            }


            match exec_result {
                Ok(cov) => {
                    if let Some(coverage) = cov {
                        if !coverage.is_empty() && !self.coverage_set.contains(&coverage) {
                            self.coverage_set.insert(coverage);
                            self.channel.send(WorkerEvent::NewCoverage).unwrap();
                        }
                    }
                },
                Err((coverage, _msg)) => {
                    if !coverage.is_empty() && !self.coverage_set.contains(&coverage) {
                        self.coverage_set.insert(coverage);
                        self.channel.send(WorkerEvent::NewCoverage).unwrap();
                    }
                    self.stats.write().unwrap().crashes += 1;
                }
            }
        }
    }
}
