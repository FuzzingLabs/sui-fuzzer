use ratatui::prelude::CrosstermBackend;
use std::io::Stdout;
use ratatui::Terminal;
use std::time::Duration;
use ratatui::{prelude::*, widgets::*};
use std::io;
use std::collections::VecDeque;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, enable_raw_mode, LeaveAlternateScreen, disable_raw_mode},
    execute,
};

use crate::fuzzer::stats::Stats;

// Data to be displayed on the tui
pub struct Ui {

    terminal: Terminal<CrosstermBackend<Stdout>>,

    // Infos (for new coverage, crashes...)
    nb_threads: u8

}

impl Ui {

    pub fn new(nb_threads: u8) -> Self {
        let terminal = Self::setup_terminal();

        Ui {
            terminal,
            nb_threads
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

    pub fn render(&mut self, stats: &Stats, events: &VecDeque<String>) -> bool {
        self.terminal.draw(|frame| {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .margin(1)
                .direction(Direction::Vertical)
                .split(frame.size());

            // Draws main block
            let main_block = Block::default().borders(Borders::ALL).title(format!("Sui Fuzzer, {} threads", self.nb_threads));
            frame.render_widget(main_block, frame.size());

            // Stats block
            let stats_block = Block::default().borders(Borders::ALL).title("Stats");
            Self::draw_stats_block(frame, chunks[0], stats);
            frame.render_widget(stats_block, chunks[0]);

            // Events block
            let events_block = Block::default().borders(Borders::ALL).title("Events");
            Self::draw_events_block(frame, chunks[0], stats, events);
            frame.render_widget(events_block, chunks[1]);

        }).unwrap();

        if event::poll(Duration::from_millis(250)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if KeyCode::Char('q') == key.code {
                    return true;
                }
            }
        }
        return false;
    }

    fn draw_stats_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        stats: &Stats,
        )
        where B: Backend {

            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .margin(1)
                .direction(Direction::Horizontal)
                .split(area);

            let text = vec![
                text::Line::from(format!("Crashes: {}", stats.crashes)),
                text::Line::from(format!("Total execs: {}", stats.execs)),
                text::Line::from(format!("Execs/s: {}", stats.execs_per_sec)),
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
                    "Worker stats:",
                    Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                    ));
            frame.render_widget(worker_stats_block, chunks[1]);
        }

    fn draw_events_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        stats: &Stats,
        events: &VecDeque<String>
        )
        where B: Backend {
            let events: Vec<ListItem> = events
                .iter()
                .map(|msg| ListItem::new(Span::raw(msg)))
                .collect();
            let events = List::new(events);
            frame.render_widget(events, area);
        }
}
