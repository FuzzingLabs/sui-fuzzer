use ratatui::{prelude::*, widgets::*};

use crate::fuzzer::stats::Stats;

pub struct GraphsWidget {
    // Coverage history for graph
    coverages: Vec<(f64, f64)>,
    // Crashes history for graph
    crashes: Vec<(f64, f64)>,
}

impl GraphsWidget {
    pub fn new() -> Self {
        Self {
            coverages: vec![],
            crashes: vec![],
        }
    }

    pub fn render<B>(&mut self, frame: &mut Frame<B>, area: Rect, stats: &Stats, index: usize)
    where
        B: Backend,
    {
        let graph_block = Block::default().borders(Borders::ALL).title(Span::styled(
            "Graphs (◄ ► to switch)",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
        // Avoid dividing by zero
        if stats.time_running == 0 {
            return;
        }
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(1)
            .direction(Direction::Horizontal)
            .split(area);

        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(chunks[0]);
        let titles = vec!["Coverage", "Crashes"]
            .iter()
            .map(|t| text::Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
            .collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Blue))
            .select(index);
        frame.render_widget(tabs, chunks[0]);

        // Adds new stats to execs_speeds vector
        self.coverages
            .push((stats.time_running as f64, stats.coverage_size as f64));
        self.crashes.push((
            stats.time_running as f64,
            (stats.crashes / stats.time_running) as f64,
        ));

        if index == 0 {
            Self::draw_graph(
                frame,
                chunks[1],
                "Coverage",
                Color::Yellow,
                stats,
                &self.coverages,
            );
        } else {
            Self::draw_graph(
                frame,
                chunks[1],
                "Crashes",
                Color::Red,
                stats,
                &self.crashes,
            );
        }

        frame.render_widget(graph_block, area);
    }

    fn draw_graph<B>(
        frame: &mut Frame<B>,
        area: Rect,
        title: &str,
        color: Color,
        stats: &Stats,
        data: &Vec<(f64, f64)>,
    ) where
        B: Backend,
    {
        let datasets = vec![Dataset::default()
            .name(title)
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(&data)];

        // Finds min and max for dynamic graph
        let min = data
            .iter()
            .fold(data[0].1, |min, &x| if x.1 < min { x.1 } else { min });
        let max = data
            .iter()
            .fold(data[0].1, |max, &x| if x.1 > max { x.1 } else { max });

        // Bindings for graph labels
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
                    .bounds([0.0, stats.time_running as f64]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![Span::from("0"), binding_mid, binding_max])
                    .bounds([0.0, max]),
            );
        frame.render_widget(chart, area);
    }
}
