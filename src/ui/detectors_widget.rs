use ratatui::{prelude::*, widgets::*};

pub struct DetectorWidget {}

impl DetectorWidget {

    pub fn new() -> Self {
        Self {}
    }

    pub fn render<B>(&self, frame: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let global_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
            "Detectors:",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ));
        let paragraph = Paragraph::new(vec![Line::from("")])
            .block(global_stats_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
}
