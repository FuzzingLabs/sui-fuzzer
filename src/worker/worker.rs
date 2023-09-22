use bichannel::Channel;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;

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
        loop {
            let exec_result = self.runner.execute();

            self.stats.write().unwrap().execs += 1;

            match exec_result {
                Ok(cov) => {
                    if let Some(coverage) = cov {
                        if !coverage.is_empty() && !self.coverage_set.contains(&coverage) {
                            self.coverage_set.insert(coverage);
                            self.channel.send(WorkerEvent::NewCoverage).unwrap();
                        }
                    }
                },
                Err(_msg) => {
                    self.stats.write().unwrap().crashes += 1;
                }
            }
        }
    }
}
