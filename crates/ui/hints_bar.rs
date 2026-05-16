use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::Frame;

use crate::app::App;
use crate::keyhints;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
	let focused_slot = app.pane_layout.focused_slot(app.focus);
	let hints = keyhints::hints_for(app.mode, focused_slot);

	let mut spans = vec![Span::raw(" ")];
	for (hint_index, hint) in hints.iter().enumerate() {
		if hint_index > 0 {
			spans.push(Span::styled("  •  ", app.theme.hints_separator));
		}
		spans.push(Span::styled(hint.key.clone(), app.theme.hints_key));
		spans.push(Span::raw(" "));
		spans.push(Span::styled(hint.label.clone(), app.theme.hints_bar));
	}
	let line = Line::from(spans);
	frame.render_widget(line, area);
}
