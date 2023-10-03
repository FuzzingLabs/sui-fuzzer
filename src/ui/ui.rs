use ratatui::prelude::CrosstermBackend;
use std::io::Stdout;
use ratatui::Terminal;
use std::time::Duration;
use ratatui::{prelude::*, widgets::*};
use std::io;
use std::sync::{RwLock, Arc};
use std::collections::VecDeque;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, enable_raw_mode, LeaveAlternateScreen, disable_raw_mode},
    execute,
};

use crate::fuzzer::stats::Stats;
use crate::ui::utils::create_event_item;

pub struct UiEventData {
    pub time: time::Duration,
    pub message: String
}

pub enum UiEvent {
    NewCoverage(UiEventData),
    NewCrash(UiEventData),
}

// Data to be displayed on the tui
pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    // Infos (for new coverage, crashes...)
    nb_threads: u8,
    // Idx of displayed thread static
    threads_stats_idx: usize,
    // Execs speeds history for graph
    execs_speeds: Vec<(f64, f64)>
}

impl Ui {

    pub fn new(nb_threads: u8) -> Self {
        let terminal = Self::setup_terminal();

        Ui {
            terminal,
            nb_threads,
            threads_stats_idx: 0,
            execs_speeds: vec![]
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

    pub fn render(&mut self, stats: &Stats, events: &VecDeque<UiEvent>, threads_stats: &Vec<Arc<RwLock<Stats>>>) -> bool {
        self.terminal.draw(|frame| {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                .margin(1)
                .direction(Direction::Vertical)
                .split(frame.size());

            // Draws main block
            let main_block = Block::default().borders(Borders::ALL).title(format!("Sui Fuzzer, {} threads (q to quit)", self.nb_threads));
            frame.render_widget(main_block, frame.size());

            // Stats block
            let stats_block = Block::default().borders(Borders::ALL).title("Stats");
            Self::draw_stats_block(frame, chunks[0], stats, self.threads_stats_idx, threads_stats, &mut self.execs_speeds);
            frame.render_widget(stats_block, chunks[0]);

            // Events block
            let events_block = Block::default().borders(Borders::ALL).title(Span::styled(
                    "Events",
                    Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
                    ));
            Self::draw_events_block(frame, chunks[1], stats, events);
            frame.render_widget(events_block, chunks[1]);

        }).unwrap();

        if event::poll(Duration::from_millis(250)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if KeyCode::Char('q') == key.code {
                    return true;
                }
                // Inputs for worker stats
                if KeyCode::Char('l') == key.code {
                    self.threads_stats_idx = 
                        if self.threads_stats_idx >= 1 { (self.threads_stats_idx - 1).into() } else { (self.nb_threads - 1).into() }  
                }
                if KeyCode::Char('r') == key.code {
                    self.threads_stats_idx = 
                        if (self.threads_stats_idx + 1) < self.nb_threads as usize { (self.threads_stats_idx + 1).into() } else { 0 }  
                }
            }
        }
        return false;
    }

    fn draw_stats_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        stats: &Stats,
        threads_stats_idx: usize,
        threads_stats: &Vec<Arc<RwLock<Stats>>>,
        execs_speeds: &mut Vec<(f64, f64)>
        )
        where B: Backend {

            let chunks = Layout::default()
                .constraints([Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(60)].as_ref())
                .margin(1)
                .direction(Direction::Horizontal)
                .split(area);

            let duration = time::Duration::seconds(stats.secs_since_last_cov.try_into().unwrap());
            let text = vec![
                text::Line::from(format!("Crashes: {}", stats.crashes)),
                text::Line::from(format!("Total execs: {}", stats.execs)),
                text::Line::from(format!("Execs/s: {}", stats.execs_per_sec)),
                text::Line::from(format!("Last coverage update: {}d {}h {}m {}s",
                                         duration.whole_days(),
                                         duration.whole_hours(),
                                         duration.whole_minutes(),
                                         duration.whole_seconds())),
            ];
            let global_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
                    "Globals stats:",
                    Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                    ));
            let paragraph = Paragraph::new(text).block(global_stats_block).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, chunks[0]);


            let worker_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
                    format!("Worker {} stats: (l/r to switch)", threads_stats_idx),
                    Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                    ));
            Self::draw_thread_stats_block(frame, chunks[1], &threads_stats[threads_stats_idx]);
            frame.render_widget(worker_stats_block, chunks[1]);

            let graph_block = Block::default().borders(Borders::ALL).title(Span::styled(
                    "Execs speeds",
                    Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
                    ));
            Self::draw_graph_block(frame, chunks[2], stats, execs_speeds);
            frame.render_widget(graph_block, chunks[2]);
        }

    fn draw_thread_stats_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        stats: &Arc<RwLock<Stats>>
        )
        where B: Backend {

            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .margin(1)
                .direction(Direction::Horizontal)
                .split(area);

            let text = vec![
                text::Line::from(format!("Crashes: {}", stats.read().unwrap().crashes)),
                text::Line::from(format!("Total execs: {}", stats.read().unwrap().execs)),
                text::Line::from(format!("Execs/s: {}", stats.read().unwrap().execs_per_sec)),
            ];
            let global_stats_block = Block::default();
            let paragraph = Paragraph::new(text).block(global_stats_block).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, chunks[0]);
    }

    fn draw_events_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        _stats: &Stats,
        events: &VecDeque<UiEvent>
        )
        where B: Backend {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .margin(1)
                .direction(Direction::Horizontal)
                .split(area);

            let events: Vec<ListItem> = events
                .iter()
                .map(|event| {
                    match event {
                        UiEvent::NewCoverage(data) => {
                            let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
                            let event_type = "COVERAGE".to_string();
                            create_event_item(data.time, style, event_type, data.message.clone())
                        },
                        UiEvent::NewCrash(data) => {
                            let style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
                            let event_type = "CRASH".to_string();
                            create_event_item(data.time, style, event_type, data.message.clone())
                        },
                    }
                }
            )
                .collect();
            let events = List::new(events).start_corner(Corner::BottomLeft);
            frame.render_widget(events, chunks[0]);
        }

    fn draw_graph_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        stats: &Stats,
        execs_speeds: &mut Vec<(f64, f64)>
        )
        where B: Backend {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(1)
                .direction(Direction::Horizontal)
                .split(area);

            execs_speeds.push((stats.time_running as f64, stats.execs_per_sec as f64));

            let min = execs_speeds.iter().fold(execs_speeds[0].1, |min, &x| if x.1 < min { x.1 } else { min });
            let max = execs_speeds.iter().fold(execs_speeds[0].1, |max, &x| if x.1 > max { x.1 } else { max });

            let datasets = vec![
                Dataset::default()
                    // .name("Execs speeds")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Yellow))
                    .data(&execs_speeds),
            ];


            let binding1 = (max as u64).to_string();
            let binding_max = binding1.bold();
            let binding2 = ((max / 2.0) as u64).to_string();
            let binding_mid = binding2.bold();
            let binding3 = (min as u64).to_string();
            let binding_min = binding3.bold();
            let chart = Chart::new(datasets)
                .x_axis(
                    Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    // .labels(x_labels)
                    .bounds([0.0, stats.time_running as f64]),
                    )
                .y_axis(
                    Axis::default()
                    .title("Execs/s")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![binding_min, binding_mid, binding_max])
                    .bounds([min, max]),
                    );
            frame.render_widget(chart, chunks[0]);
        }
}
