use std::{
    collections::HashSet, sync::{Arc, RwLock}, time::Instant
};

use bichannel::Channel;

use crate::{
    detector::detector::{AvailableDetector, Detector},
    fuzzer::{coverage::Coverage, stats::Stats},
    mutator::{mutator::Mutator, rng::Rng},
    runner::runner::Runner,
    worker::worker::WorkerEvent,
};

use super::worker::Worker;

#[allow(dead_code)]
const STATE_INIT_POSTFIX: &str = "init";

pub struct StatefulWorker {
    channel: Channel<WorkerEvent, WorkerEvent>,
    stats: Arc<RwLock<Stats>>,
    runner: Box<dyn Runner>,
    mutator: Box<dyn Mutator>,
    rng: Rng,
    detectors: Vec<Box<dyn Detector>>,
    seed: u64,
}

impl StatefulWorker {
    pub fn new(
        channel: Channel<WorkerEvent, WorkerEvent>,
        stats: Arc<RwLock<Stats>>,
        _coverage_set: HashSet<Coverage>,
        runner: Box<dyn Runner>,
        mutator: Box<dyn Mutator>,
        seed: u64,
        _execs_before_cov_update: u64,
        _available_detectors: Option<Vec<AvailableDetector>>,
    ) -> Self {
        StatefulWorker {
            channel,
            stats,
            runner,
            mutator,
            seed,
            rng: Rng {
                seed,
                exp_disabled: false,
            },
            detectors: vec![],
        }
    }
}

impl Worker for StatefulWorker {
    fn run(&mut self) {
        // Input initialization
        let mut inputs = self.runner.get_target_parameters();

        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();
        let mut sec_elapsed = 0;

        loop {
            let _exec_result = self.runner.execute(inputs.clone());

            self.stats.write().unwrap().execs += 1;

            // Calculate execs_per_sec
            if execs_per_sec_timer.elapsed().as_secs() >= 1 {
                execs_per_sec_timer = Instant::now();
                sec_elapsed += 1;
                let tmp = self.stats.read().unwrap().execs;
                self.stats.write().unwrap().secs_since_last_cov += 1;
                self.stats.write().unwrap().execs_per_sec = tmp / sec_elapsed;
            }

            // Mutate inputs
            inputs = self.mutator.mutate(&inputs, 4);
        }
    }
}
