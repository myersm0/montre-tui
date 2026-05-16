use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::Frame;

use crate::theme::Theme;

pub struct KeyHint {
	pub key: String,
	pub label: String,
}

impl KeyHint {
	pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
		Self {
			key: key.into(),
			label: label.into(),
		}
	}
}

pub fn draw_hints_bar(frame: &mut Frame, area: Rect, hints: &[KeyHint], theme: &Theme) {
	let mut spans = vec![Span::raw(" ")];
	for (hint_index, hint) in hints.iter().enumerate() {
		if hint_index > 0 {
			spans.push(Span::styled("  •  ", theme.hints_separator));
		}
		spans.push(Span::styled(hint.key.clone(), theme.hints_key));
		spans.push(Span::raw(" "));
		spans.push(Span::styled(hint.label.clone(), theme.hints_bar));
	}
	let line = Line::from(spans);
	frame.render_widget(line, area);
}
