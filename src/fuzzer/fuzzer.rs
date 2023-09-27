use bichannel::Channel;
use std::sync::RwLock;
use std::sync::Arc;
use std::time::Instant;
use std::collections::VecDeque;
use time::Duration;

use crate::fuzzer::stats::Stats;
use crate::fuzzer::config::Config;
use crate::worker::worker::{Worker, WorkerEvent};
use crate::ui::ui::Ui;

// Sui specific imports
use crate::runner::sui_runner::SuiRunner;
use crate::mutator::sui_mutator::SuiMutator;

pub struct Fuzzer {
    // Fuzzer configuration
    config: Config,
    // Thread specific stats
    threads_stats: Vec<Arc<RwLock<Stats>>>,
    // Channel to communicate with each threads
    channels: Vec<Channel<u8, WorkerEvent>>,
    // Global stats mostly for ui
    global_stats: Stats,

    // The user interface
    ui: Option<Ui>
}

impl Fuzzer {
    
    pub fn new(config: Config) -> Self {
        let nb_threads = config.nb_threads;
        Fuzzer {
            config,
            threads_stats: vec![],
            channels: vec![],
            global_stats: Stats::new(),
            ui: Some(Ui::new(nb_threads))
        }
    }

    fn start_threads(&mut self) {
        for i in 0..self.config.nb_threads {
            // Creates the communication channel for the fuzzer and worker sides
            let (fuzzer, worker) = bichannel::channel::<u8, WorkerEvent>();
            self.channels.push(fuzzer);
            let stats = Arc::new(RwLock::new(Stats::new()));
            self.threads_stats.push(stats.clone());
            // Change here the runner you want to create
            if let Some(parameter) = &self.config.runner_parameter {
                let runner = Box::new(SuiRunner::new(&parameter.clone()));
                let seed = self.config.seed.unwrap() + (i as u64);
                let mutator = Box::new(SuiMutator::new(seed, 11));
                std::thread::spawn(move || {
                    // Creates generic worker and starts it
                    let mut w = Worker::new(worker, stats, runner, mutator, seed);
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
                self.global_stats.execs_per_sec = self.global_stats.execs_per_sec / self.global_stats.time_running;
            }

            // Check channels for new data
            for chan in &self.channels {
                if let Ok(event) = chan.try_recv() {
                    match event {
                        WorkerEvent::NewCoverage(input) => {
                            let duration = Duration::seconds(self.global_stats.time_running.try_into().unwrap());
                            self.global_stats.secs_since_last_cov = 0;
                            events.push_front(String::from(format!("{}d {}h {}m {}s -> New coverage with: {}",
                                                                   duration.whole_days(),
                                                                   duration.whole_hours(),
                                                                   duration.whole_minutes(),
                                                                   duration.whole_seconds(),
                                                                   String::from_utf8_lossy(&input))));
                        },
                    }
                }
            }

            // Run ui
            if self.config.use_ui {
                if self.ui.as_mut().unwrap().render(&self.global_stats, &events, &self.threads_stats) {
                    self.ui.as_mut().unwrap().restore_terminal();
                    break;
                }
            } else {}

        }
    }

}
