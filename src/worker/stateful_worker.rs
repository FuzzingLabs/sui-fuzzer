use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
    time::Instant,
};

use bichannel::Channel;
use move_model::ty::Type;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    detector::detector::AvailableDetector,
    fuzzer::{coverage::Coverage, crash::Crash, stats::Stats},
    mutator::{mutator::Mutator, types::Type as FuzzerType},
    runner::runner::StatefulRunner,
    runner::stateless_runner::sui_runner_utils::generate_abi_from_source,
    worker::worker::WorkerEvent,
};

use super::worker::Worker;

#[allow(dead_code)]
const STATE_INIT_POSTFIX: &str = "init";

pub struct StatefulWorker {
    channel: Channel<WorkerEvent, WorkerEvent>,
    stats: Arc<RwLock<Stats>>,
    runner: Box<dyn StatefulRunner>,
    mutator: Box<dyn Mutator>,
    unique_crashes_set: HashSet<Crash>,
    // Available functions
    target_functions: Vec<FuzzerType>,
}

impl StatefulWorker {
    pub fn new(
        contract: &str,
        channel: Channel<WorkerEvent, WorkerEvent>,
        stats: Arc<RwLock<Stats>>,
        _coverage_set: HashSet<Coverage>,
        runner: Box<dyn StatefulRunner>,
        mutator: Box<dyn Mutator>,
        _execs_before_cov_update: u64,
        _available_detectors: Option<Vec<AvailableDetector>>,
        target_module: &str,
        target_functions: Vec<String>,
    ) -> Self {
        let mut functions = vec![];
        for target_function in &target_functions {
            let (parameters, _) =
                generate_abi_from_source(contract, target_module, target_function);
            functions.push(FuzzerType::Function(
                target_function.clone(),
                Self::transform_params(parameters),
                None,
            ));
        }

        StatefulWorker {
            channel,
            stats,
            runner,
            mutator,
            target_functions: functions,
            unique_crashes_set: HashSet::new(),
        }
    }

    fn transform_params(params: Vec<Type>) -> Vec<FuzzerType> {
        let mut res = vec![];
        for param in params {
            res.push(FuzzerType::from(param));
        }
        res
    }

    fn generate_call_sequence(&self, size: u32) -> Vec<FuzzerType> {
        let mut target_functions = self.target_functions.clone();
        let mut call_sequence: Vec<FuzzerType> = vec![FuzzerType::Function(
            "fuzz_check".to_string(),
            vec![
                FuzzerType::Reference(false, Box::new(FuzzerType::Struct(vec![]))),
                FuzzerType::Reference(false, Box::new(FuzzerType::Struct(vec![]))),
            ],
            None,
        )];
        call_sequence.append(&mut target_functions);
        for _ in 0..size {
            let n = self
                .mutator
                .generate_number(0, (call_sequence.len() - 1).try_into().unwrap());
            call_sequence.push(call_sequence[n as usize].clone());
        }
        call_sequence.shuffle(&mut thread_rng());
        call_sequence
    }
}

impl Worker for StatefulWorker {
    fn run(&mut self) {
        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();
        let mut sec_elapsed = 0;

        loop {
            let call_sequence = self.generate_call_sequence(5);

            // Call each function in the call sequence
            for function in call_sequence {
                // Reset function
                self.runner.set_target_function(&function);

                // Input initialization
                let mut inputs = function.as_function().unwrap().1.clone();

                // Mutate inputs
                inputs = self.mutator.mutate(&inputs, 4);

                //eprintln!("{} {:?}", function.as_function().unwrap().0, inputs);

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
                    Ok(_) => continue,
                    Err((_cov, error)) => {
                        self.stats.write().unwrap().crashes += 1;
                        let crash = Crash::new(
                            &self.runner.get_target_module(),
                            &self.runner.get_target_function().as_function().unwrap().0,
                            &inputs,
                            &error,
                        );
                        if !self.unique_crashes_set.contains(&crash) {
                            self.channel
                                .send(WorkerEvent::NewCrash(
                                    self.runner
                                        .get_target_function()
                                        .as_function()
                                        .unwrap()
                                        .0
                                        .to_string(),
                                    inputs.clone(),
                                    error,
                                ))
                                .unwrap();
                        }
                    }
                }
            }

            // Reset state
            self.runner.setup();
        }
    }
}
