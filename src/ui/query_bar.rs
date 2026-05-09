use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::Frame;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
	let line = Line::from(vec![
		Span::styled(" Query: ", app.theme.query_prompt),
		Span::styled("—", app.theme.query_input),
	]);
	frame.render_widget(line, area);
}
