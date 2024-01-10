use std::collections::VecDeque;

use ratatui::{prelude::*, widgets::*};

use super::{ui::UiEvent, utils::create_event_item};

pub struct EventsWidget {
    events: VecDeque<UiEvent>,
}

impl EventsWidget {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    pub fn render<B>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        events: &mut VecDeque<UiEvent>,
        scroll: &mut u16,
    ) where
        B: Backend,
    {
        if events.len() != 0 {
            let len = events.len() as u16;
            *scroll += 1;
            self.events.append(events);
        }

        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(1)
            .direction(Direction::Horizontal)
            .split(area);

        // Generates listitems for events
        let events: Vec<Line> = self
            .events
            .iter()
            .map(|event| match event {
                UiEvent::NewCoverage(data) => {
                    let style = Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD);
                    let event_type = "COVERAGE".to_string();
                    create_event_item(
                        data.time,
                        style,
                        event_type,
                        format!(" with input: {}", data.message),
                    )
                }
                UiEvent::NewCrash(data) => {
                    let style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
                    let error = data.error.clone().unwrap();
                    let event_type = format!("CRASH Type: {}", error).to_string();
                    create_event_item(
                        data.time,
                        style,
                        event_type,
                        format!(" with input: {}", data.message),
                    )
                }
                UiEvent::DetectorTriggered(data) => {
                    let style = Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD);
                    let event_type = format!("Detector triggered: ").to_string();
                    create_event_item(data.time, style, event_type, data.message.clone())
                }
            })
            //.rev()
            .collect();

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let content_len = events.iter().len() as u16;
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(content_len)
            .position(*scroll);

        let paragraph = Paragraph::new(events.clone())
            .scroll((*scroll, 0))
            .block(Block::new().borders(Borders::RIGHT))
            .wrap(Wrap { trim: true });

        // Rendering everything
        frame.render_widget(paragraph, chunks[0]);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
            &mut scrollbar_state,
        );
    }
}
