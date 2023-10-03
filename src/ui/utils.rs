use ratatui::{prelude::*, widgets::*};
use time::Duration;

/// Returns a ListItem with given data (used to avoid code duplicates)
pub fn create_event_item(time: Duration, style: Style, event_type: String, message: String) -> ListItem<'static> {
    ListItem::new(vec![Line::from(vec![
                                  Span::from(format!("{}d {}h {}m {}s",
                                                     time.whole_days(),
                                                     time.whole_hours(),
                                                     time.whole_minutes(),
                                                     time.whole_seconds(),
                                                     )),
                                                     ": ".into(),
                                                     Span::styled(event_type.clone(), style),
                                                     " with input: ".into(),
                                                     message.clone().into()
    ]),
    ])
}
