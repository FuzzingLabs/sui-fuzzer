use ratatui::{prelude::*, widgets::*};
use crate::{mutator::types::Parameters, mutator::types::Type};

pub struct TargetInfosWidget {
    target_module: String,
    target_function: String,
    target_parameters: Vec<Type>,
}

impl TargetInfosWidget {
    pub fn new(
        target_module: String,
        target_function: String,
        target_parameters: Vec<Type>,
    ) -> Self {
        Self {
            target_module,
            target_function,
            target_parameters,
        }
    }

    pub fn render<B>(&self, frame: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let text = vec![
            text::Line::from(vec![Span::styled("Target: ", Style::new().green())]),
            text::Line::from(format!("{}::{}", self.target_module, self.target_function)),
            text::Line::from(vec![Span::styled("Parameters: ", Style::new().green())]),
            text::Line::from(format!("{}", Parameters(self.target_parameters.to_vec()))),
        ];

        let global_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
            "Target info:",
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        ));
        let paragraph = Paragraph::new(text)
            .block(global_stats_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
}
