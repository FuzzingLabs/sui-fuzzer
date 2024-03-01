use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use crate::detector::detector::AvailableDetector;

pub struct DetectorWidget {}

impl DetectorWidget {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<B>(
        &self,
        frame: &mut Frame<B>,
        area: Rect,
        detectors: &Option<Vec<AvailableDetector>>,
    ) where
        B: Backend,
    {
        let global_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
            "Detectors:",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ));
        let mut text = vec![Line::from("No detectors !")];
        if let Some(detectors) = detectors {
            text = detectors
                .iter()
                .map(|d| Line::from(format!("{:?}", d)))
                .collect_vec();
        }
        let paragraph = Paragraph::new(text)
            .block(global_stats_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
}
