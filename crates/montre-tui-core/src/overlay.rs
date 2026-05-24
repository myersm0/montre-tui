use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::theme::Theme;

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
	let vertical = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Percentage((100 - percent_y) / 2),
			Constraint::Percentage(percent_y),
			Constraint::Percentage((100 - percent_y) / 2),
		])
		.split(area);
	Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Percentage((100 - percent_x) / 2),
			Constraint::Percentage(percent_x),
			Constraint::Percentage((100 - percent_x) / 2),
		])
		.split(vertical[1])[1]
}

pub fn draw_help(
	frame: &mut Frame,
	area: Rect,
	theme: &Theme,
	title: &str,
	entries: &[(&str, &str)],
) {
	let overlay_area = centered_rect(60, 70, area);
	frame.render_widget(Clear, overlay_area);

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(theme.overlay_border.border_type)
		.border_style(theme.overlay_border.style)
		.style(theme.overlay_background)
		.title(Span::styled(format!(" {} ", title), theme.overlay_title));
	let inner = block.inner(overlay_area);
	frame.render_widget(block, overlay_area);

	let lines: Vec<Line> = entries
		.iter()
		.map(|(keys, description)| {
			Line::from(vec![
				Span::styled(format!("  {:<14}", keys), theme.hints_key),
				Span::styled(description.to_string(), theme.hints_bar),
			])
		})
		.collect();

	frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

pub fn draw_shutdown(frame: &mut Frame, area: Rect, reason: &str, theme: &Theme) {
	let overlay_area = centered_rect(50, 20, area);
	frame.render_widget(Clear, overlay_area);

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(theme.overlay_border.border_type)
		.border_style(theme.overlay_border.style)
		.style(theme.overlay_background)
		.title(Span::styled(" daemon shutdown ", theme.overlay_title));
	let inner = block.inner(overlay_area);
	frame.render_widget(block, overlay_area);

	let lines = vec![
		Line::from(""),
		Line::from(Span::styled(format!("  reason: {}", reason), theme.error)),
		Line::from(""),
		Line::from(Span::styled("  exiting...".to_string(), theme.text_subtle)),
	];
	frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}
