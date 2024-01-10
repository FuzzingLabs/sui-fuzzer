use ratatui::{prelude::*, widgets::*};

#[cfg(target_os = "linux")]
use memory_stats::memory_stats;

use crate::fuzzer::stats::Stats;

pub struct GlobalStatsWidget {
    seed: u64,
    max_coverage: usize,
}

impl GlobalStatsWidget {
    pub fn new(seed: u64, max_coverage: usize) -> Self {
        Self { seed, max_coverage }
    }

    pub fn render<B>(&self, frame: &mut Frame<B>, area: Rect, stats: &Stats)
    where
        B: Backend,
    {
        let duration = time::Duration::seconds(stats.secs_since_last_cov.try_into().unwrap());
        let running_duration = time::Duration::seconds(stats.time_running.try_into().unwrap());

        let mut text = vec![
            text::Line::from(format!("Seed: {}", self.seed)),
            text::Line::from(format!("Crashes: {}", stats.crashes)),
            text::Line::from(format!("Unique crashes: {}", stats.unique_crashes)),
            text::Line::from(format!("Total execs: {}", stats.execs)),
            text::Line::from(format!("Execs/s: {}", stats.execs_per_sec)),
            text::Line::from(format!(
                "Coverage: {}/{}",
                stats.coverage_size, self.max_coverage
            )),
            text::Line::from(format!(
                "Running for: {}d {}h {}m {}s",
                running_duration.whole_days(),
                running_duration.whole_hours(),
                running_duration.whole_minutes(),
                running_duration.whole_seconds()
            )),
            text::Line::from(format!(
                "Last coverage update: {}d {}h {}m {}s",
                duration.whole_days(),
                duration.whole_hours(),
                duration.whole_minutes(),
                duration.whole_seconds()
            )),
        ];
        // The crate for the memory doesn't work on mac
        if cfg!(target_os = "linux") {
            // Gets memory usage
            let mut mem = 0;
            if let Some(usage) = memory_stats() {
                mem = usage.virtual_mem;
            }
            text.push(text::Line::from(format!(
                "Memory usage: {} MB",
                mem / 1000000
            )));
        }

        let global_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
            "Fuzzing statistics:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
        let paragraph = Paragraph::new(text)
            .block(global_stats_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
}
