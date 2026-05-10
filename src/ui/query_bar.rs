use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::Frame;

use crate::app::{App, Mode};

const PROMPT: &str = " Query: ";

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
	let in_query_entry = matches!(app.mode, Mode::QueryEntry);

	let displayed: &str = if in_query_entry {
		&app.query_bar.text
	} else if app.query_bar.text.is_empty() {
		"—"
	} else {
		&app.query_bar.text
	};

	let line = Line::from(vec![
		Span::styled(PROMPT, app.theme.query_prompt),
		Span::styled(displayed, app.theme.query_input),
	]);
	frame.render_widget(line, area);

	if in_query_entry {
		let column = area.x + PROMPT.len() as u16 + app.query_bar.cursor as u16;
		frame.set_cursor_position((column, area.y));
	}
}
