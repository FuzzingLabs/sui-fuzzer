use std::sync::{Arc, RwLock};
use ratatui::{prelude::*, widgets::*};
use crate::fuzzer::stats::Stats;

pub struct WorkerStatsWidget {}

impl WorkerStatsWidget {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<B>(
        &self,
        frame: &mut Frame<B>,
        area: Rect,
        index: usize,
        stats: &Arc<RwLock<Stats>>,
    ) where
        B: Backend,
    {
        let worker_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
            format!("Worker {} stats: (l/r to switch)", index),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(1)
            .direction(Direction::Horizontal)
            .split(area);

        let text = vec![
            text::Line::from(format!("Crashes: {}", stats.read().unwrap().crashes)),
            text::Line::from(format!("Total execs: {}", stats.read().unwrap().execs)),
            text::Line::from(format!("Execs/s: {}", stats.read().unwrap().execs_per_sec)),
        ];
        let global_stats_block = Block::default();
        let paragraph = Paragraph::new(text)
            .block(global_stats_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, chunks[0]);

        frame.render_widget(worker_stats_block, area);
    }
}
