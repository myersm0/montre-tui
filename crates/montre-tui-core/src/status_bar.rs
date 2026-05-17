use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::Frame;

use crate::theme::Theme;

pub struct StatusBarContent<'a> {
	pub corpus_name: &'a str,
	pub component: Option<&'a str>,
	pub selection: Option<&'a str>,
	pub coupler_info: Option<&'a str>,
	pub connected: bool,
}

pub fn draw_status_bar(frame: &mut Frame, area: Rect, content: &StatusBarContent, theme: &Theme) {
	let dot_style = if content.connected {
		theme.status_dot_connected
	} else {
		theme.status_dot_disconnected
	};

	let mut spans: Vec<Span> = Vec::new();
	spans.push(Span::raw(" "));
	spans.push(Span::styled("●", dot_style));
	spans.push(Span::styled(" ", theme.status_bar));
	spans.push(Span::styled(content.corpus_name.to_string(), theme.status_bar));

	let separator = Span::styled(" │ ", theme.status_bar);
	spans.push(separator.clone());
	spans.push(Span::styled(
		content.component.unwrap_or("—").to_string(),
		theme.status_bar,
	));

	spans.push(separator.clone());
	spans.push(Span::styled(
		content.selection.unwrap_or("—").to_string(),
		theme.status_bar,
	));

	spans.push(separator);
	spans.push(Span::styled(
		content.coupler_info.unwrap_or("—").to_string(),
		theme.status_bar,
	));
	spans.push(Span::styled(" ", theme.status_bar));

	let line = Line::from(spans);
	frame.render_widget(line, area);
}
