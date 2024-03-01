use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use ratatui::{prelude::*, widgets::*};
use std::collections::VecDeque;
use std::io;
use std::io::Stdout;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use crate::{detector::detector::AvailableDetector, fuzzer::stats::Stats};
use crate::{fuzzer::error::Error, mutator::types::Type};
use super::{events_widget::EventsWidget, stats_widget::StatsWidget};

#[derive(Debug, Clone)]
pub struct UiEventData {
    pub time: time::Duration,
    pub message: String,
    pub error: Option<Error>,
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    NewCoverage(UiEventData),
    NewCrash(UiEventData),
    DetectorTriggered(UiEventData),
}

// Data to be displayed on the tui
pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    // Index for graph tabs
    tab_index: usize,
    // Infos (for new coverage, coverages...)
    nb_threads: u8,
    // Idx of displayed thread stats
    threads_stats_idx: usize,
    scroll: u16,
    // Seed of the mutator
    seed: u64,
    stats_widget: Option<StatsWidget>,
    events_widget: EventsWidget,
}

impl Ui {
    pub fn new(nb_threads: u8, seed: u64) -> Self {
        // Setup panic hook
        Self::initialize_panic_handler();
        let terminal = Self::setup_terminal();

        Ui {
            terminal,
            nb_threads,
            seed,
            tab_index: 0,
            threads_stats_idx: 0,
            scroll: 0,
            stats_widget: None,
            events_widget: EventsWidget::new(),
        }
    }

    fn setup_terminal() -> Terminal<CrosstermBackend<Stdout>> {
        let mut stdout = io::stdout();
        enable_raw_mode().expect("failed to enable raw mode");
        execute!(stdout, EnterAlternateScreen).expect("unable to enter alternate screen");
        Terminal::new(CrosstermBackend::new(stdout)).expect("creating terminal failed")
    }

    pub fn restore_terminal(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).unwrap();
        self.terminal.show_cursor().unwrap();
    }

    pub fn initialize_panic_handler() {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }

    pub fn set_target_infos(
        &mut self,
        target_module: &str,
        target_function: &str,
        target_parameters: &Vec<Type>,
        max_coverage: usize,
    ) {
        self.stats_widget = Some(StatsWidget::new(
            self.seed,
            max_coverage,
            target_module,
            target_function,
            target_parameters,
        ))
    }

    pub fn render(
        &mut self,
        stats: &Stats,
        mut events: &mut VecDeque<UiEvent>,
        threads_stats: &Vec<Arc<RwLock<Stats>>>,
        detectors: &Option<Vec<AvailableDetector>>,
        use_state: bool
    ) -> bool {
        self.terminal
            .draw(|frame| {
                let chunks = Layout::default()
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .margin(1)
                    .direction(Direction::Vertical)
                    .split(frame.size());

                // Draws main block
                let main_block = Block::default().borders(Borders::ALL).title(format!(
                    "Sui Fuzzer, {} threads (q to quit)",
                    self.nb_threads
                ));
                frame.render_widget(main_block, frame.size());

                // Stats block
                let stats_block = Block::default().borders(Borders::ALL).title("Stats");
                if let Some(ref mut widget) = self.stats_widget {
                    widget.render(
                        frame,
                        chunks[0],
                        stats,
                        self.tab_index,
                        self.threads_stats_idx,
                        threads_stats,
                        detectors,
                        use_state
                    );
                }
                frame.render_widget(stats_block, chunks[0]);

                // Events block
                let events_block = Block::default().borders(Borders::ALL).title(Span::styled(
                    "Events (▲ ▼ to scroll)",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ));
                self.events_widget
                    .render(frame, chunks[1], &mut events, &mut self.scroll);
                frame.render_widget(events_block, chunks[1]);
            })
            .unwrap();

        if event::poll(Duration::from_millis(250)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    // q to quit the fuzzer
                    KeyCode::Char('q') => return true,
                    // Inputs for worker stats
                    KeyCode::Char('l') => {
                        self.threads_stats_idx = if self.threads_stats_idx >= 1 {
                            (self.threads_stats_idx - 1).into()
                        } else {
                            (self.nb_threads - 1).into()
                        }
                    }
                    KeyCode::Char('r') => {
                        self.threads_stats_idx =
                            if (self.threads_stats_idx + 1) < self.nb_threads as usize {
                                (self.threads_stats_idx + 1).into()
                            } else {
                                0
                            }
                    }
                    // Inputs for graphs
                    KeyCode::Left => self.tab_index = if self.tab_index == 0 { 1 } else { 0 },
                    KeyCode::Right => self.tab_index = if self.tab_index == 1 { 0 } else { 1 },
                    // Input for events
                    KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                    KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                    _ => return false,
                }
            }
        }
        return false;
    }
}
