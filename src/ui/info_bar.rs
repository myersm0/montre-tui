use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
	let corpus_name = app
		.corpus_path
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("?");
	let text = format!(" {} │ — │ — │ — │ — ", corpus_name);
	let line = Line::from(text).style(app.theme.status_bar);
	frame.render_widget(line, area);
}
