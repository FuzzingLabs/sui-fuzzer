use bichannel::Channel;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;
use time::Duration;

use crate::fuzzer::config::Config;
use crate::fuzzer::coverage::Coverage;
use crate::fuzzer::stats::Stats;
use crate::mutator::types::Parameters;
use crate::mutator::types::Type;
use crate::runner::runner::Runner;
use crate::ui::ui::{Ui, UiEvent, UiEventData};
use crate::worker::worker::{Worker, WorkerEvent};
// Sui specific imports
use crate::mutator::sui_mutator::SuiMutator;
use crate::runner::sui_runner::SuiRunner;

use super::crash::Crash;
use super::fuzzer_utils::write_crashfile;

pub struct Fuzzer {
    // Fuzzer configuration
    config: Config,
    // Thread specific stats
    threads_stats: Vec<Arc<RwLock<Stats>>>,
    // Channel to communicate with each threads
    channels: Vec<Channel<WorkerEvent, WorkerEvent>>,
    // Global stats mostly for ui
    global_stats: Stats,
    // Global coverage
    coverage_set: HashSet<Coverage>,
    // Unique crashes set
    unique_crashes_set: HashSet<Crash>,
    // The user interface
    ui: Option<Ui>,
    // The function to target in the contract
    target_module: String,
    // The function to target in the contract
    target_function: String,
    // Parameters of the target function
    target_parameters: Vec<Type>,
}

impl Fuzzer {
    pub fn new(config: Config, target_module: &str, target_function: &str) -> Self {
        let nb_threads = config.nb_threads;
        Fuzzer {
            config,
            threads_stats: vec![],
            channels: vec![],
            global_stats: Stats::new(),
            coverage_set: HashSet::new(),
            unique_crashes_set: HashSet::new(),
            ui: Some(Ui::new(nb_threads)),
            target_module: String::from(target_module),
            target_function: String::from(target_function),
            target_parameters: vec![],
        }
    }

    fn start_threads(&mut self) {
        for i in 0..self.config.nb_threads {
            // Creates the communication channel for the fuzzer and worker sides
            let (fuzzer, worker) = bichannel::channel::<WorkerEvent, WorkerEvent>();
            self.channels.push(fuzzer);
            let stats = Arc::new(RwLock::new(Stats::new()));
            self.threads_stats.push(stats.clone());
            // Change here the runner you want to create
            if let Some(parameter) = &self.config.contract_file {
                // Creates the sui runner with the runner parameter found in the config
                let runner = Box::new(SuiRunner::new(
                    &parameter.clone(),
                    &self.target_module,
                    &self.target_function,
                ));
                self.target_parameters = runner.get_target_parameters();
                // Increment seed so that each worker doesn't do the same thing
                let seed = self.config.seed.unwrap() + (i as u64);
                let execs_before_cov_update = self.config.execs_before_cov_update;
                let mutator = Box::new(SuiMutator::new(seed, 12));
                let _ = std::thread::Builder::new()
                    .name(format!("Worker {}", i).to_string())
                    .spawn(move || {
                        // Creates generic worker and starts it
                        let mut w = Worker::new(
                            worker,
                            stats,
                            runner,
                            mutator,
                            seed,
                            execs_before_cov_update,
                        );
                        w.run();
                    });
            }
        }
    }

    fn get_global_execs(&self) -> u64 {
        let mut sum: u64 = 0;
        for i in 0..self.config.nb_threads {
            sum += self.threads_stats[i as usize].read().unwrap().execs;
        }
        sum
    }

    fn get_global_crashes(&self) -> u64 {
        let mut sum: u64 = 0;
        for i in 0..self.config.nb_threads {
            sum += self.threads_stats[i as usize].read().unwrap().crashes;
        }
        sum
    }

    pub fn run(&mut self) {
        // Init workers
        self.start_threads();

        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();

        let mut events = VecDeque::new();

        if let Some(ui) = &mut self.ui {
            ui.set_target_infos(
                &self.target_module,
                &self.target_function,
                &self.target_parameters,
            );
        }

        loop {
            // Sum execs
            self.global_stats.execs = self.get_global_execs();
            self.global_stats.crashes = self.get_global_crashes();

            // Calculate execs_per_sec
            if execs_per_sec_timer.elapsed().as_secs() >= 1 {
                execs_per_sec_timer = Instant::now();
                self.global_stats.execs_per_sec = self.global_stats.execs;
                self.global_stats.time_running += 1;
                self.global_stats.secs_since_last_cov += 1;
                self.global_stats.execs_per_sec =
                    self.global_stats.execs_per_sec / self.global_stats.time_running;
            }

            // Checks channels for new data
            for chan in &self.channels {
                if let Ok(event) = chan.try_recv() {
                    // Creates duration used for the ui
                    let duration =
                        Duration::seconds(self.global_stats.time_running.try_into().unwrap());
                    match event {
                        WorkerEvent::CoverageUpdateRequest(coverage_set) => {
                            // Gets diffrences between the two coverage sets
                            let binding = self.coverage_set.clone();
                            let differences_with_main_thread: HashSet<_> =
                                self.coverage_set.difference(&coverage_set).collect();
                            let differences_with_worker: HashSet<_> =
                                coverage_set.difference(&binding).collect();
                            let mut tmp = HashSet::new();
                            for diff in &differences_with_main_thread.clone() {
                                tmp.insert(diff.to_owned().clone());
                            }
                            // Updates sets
                            if differences_with_main_thread.len() > 0 {
                                chan.send(WorkerEvent::CoverageUpdateResponse(tmp)).unwrap();
                            }
                            // Adds all the coverage to the main coverage_set
                            for diff in &differences_with_worker {
                                if !self.coverage_set.contains(diff) {
                                    self.coverage_set.insert(diff.to_owned().clone());
                                    self.global_stats.secs_since_last_cov = 0;
                                    self.global_stats.coverage_size += 1;
                                    events.push_front(UiEvent::NewCoverage(UiEventData {
                                        time: duration,
                                        message: format!("{}", Parameters(diff.inputs.clone())),
                                        error: None,
                                    }));
                                }
                            }
                        }
                        WorkerEvent::NewCrash(inputs, error) => {
                            let crash = Crash::new(&self.target_function, &inputs, &error);
                            if !self.unique_crashes_set.contains(&crash) {
                                write_crashfile(&self.config.crashes_dir, crash.clone());
                                self.global_stats.unique_crashes += 1;
                                self.unique_crashes_set.insert(crash.clone());
                            }
                            events.push_front(UiEvent::NewCrash(UiEventData {
                                time: duration,
                                message: format!("{}", Parameters(inputs)),
                                error: Some(error),
                            }));
                        }
                        _ => unimplemented!(),
                    }
                }
            }

            // Run ui
            if self.config.use_ui {
                if self.ui.as_mut().unwrap().render(
                    &self.global_stats,
                    &events,
                    &self.threads_stats,
                ) {
                    self.ui.as_mut().unwrap().restore_terminal();
                    eprintln!("Quitting...");
                    break;
                }
            } else {
                // TODO Implement simple println ui
            }
        }
    }
}
