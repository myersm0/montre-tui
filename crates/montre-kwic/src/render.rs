use anyhow::Result;
use montre_tui_core::key_hint::{draw_hints_bar, KeyHint};
use montre_tui_core::status_bar::{draw_status_bar, StatusBarContent};
use montre_tui_core::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::daemon_access::DaemonAccess;

pub enum Mode {
	Normal,
	Help,
	ShuttingDown { reason: String },
}

pub struct ViewState<'a> {
	pub mode: &'a Mode,
	pub connected: bool,
	pub theme: &'a Theme,
}

pub fn draw(frame: &mut Frame, access: &DaemonAccess, view: &ViewState) -> Result<()> {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(1),
			Constraint::Min(1),
			Constraint::Length(1),
			Constraint::Length(1),
		])
		.split(frame.area());

	let query_bar_area = layout[0];
	let body_area = layout[1];
	let hints_area = layout[2];
	let status_area = layout[3];

	draw_query_bar(frame, query_bar_area, view.theme);
	draw_kwic_pane(frame, body_area, view.theme);
	draw_hints_bar(frame, hints_area, &kwic_hints(), view.theme);
	draw_kwic_status(frame, status_area, access, view);

	match view.mode {
		Mode::Normal => {}
		Mode::Help => draw_help_overlay(frame, frame.area(), view.theme),
		Mode::ShuttingDown { reason } => draw_shutdown_overlay(frame, frame.area(), reason, view.theme),
	}

	Ok(())
}

fn draw_query_bar(frame: &mut Frame, area: Rect, theme: &Theme) {
	let line = Line::from(vec![Span::styled(" » ", theme.hints_key)]);
	frame.render_widget(line, area);
}

fn draw_kwic_pane(frame: &mut Frame, area: Rect, theme: &Theme) {
	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(theme.pane_border_active.border_type)
		.border_style(theme.pane_border_active.style)
		.title(Span::styled(" KWIC ", theme.pane_title));

	let inner = block.inner(area);
	frame.render_widget(block, area);

	let placeholder = Line::from(Span::styled("No query yet.", theme.hints_bar));
	frame.render_widget(Paragraph::new(Text::from(placeholder)), inner);
}

fn draw_kwic_status(frame: &mut Frame, area: Rect, access: &DaemonAccess, view: &ViewState) {
	let content = StatusBarContent {
		corpus_name: access.corpus_info.name.as_str(),
		component: None,
		selection: None,
		coupler_info: None,
		connected: view.connected,
	};
	draw_status_bar(frame, area, &content, view.theme);
}

fn draw_help_overlay(frame: &mut Frame, area: Rect, theme: &Theme) {
	let overlay_area = centered_rect(60, 70, area);
	frame.render_widget(Clear, overlay_area);

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(theme.overlay_border.border_type)
		.border_style(theme.overlay_border.style)
		.title(Span::styled(" kwic help ", theme.overlay_title));
	let inner = block.inner(overlay_area);
	frame.render_widget(block, overlay_area);

	let entries = [
		("?", "toggle help"),
		("q  Esc", "quit"),
	];
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

fn draw_shutdown_overlay(frame: &mut Frame, area: Rect, reason: &str, theme: &Theme) {
	let overlay_area = centered_rect(50, 20, area);
	frame.render_widget(Clear, overlay_area);

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(theme.overlay_border.border_type)
		.border_style(theme.overlay_border.style)
		.title(Span::styled(" daemon shutdown ", theme.overlay_title));
	let inner = block.inner(overlay_area);
	frame.render_widget(block, overlay_area);

	let lines = vec![
		Line::from(""),
		Line::from(Span::styled(format!("  reason: {}", reason), theme.error)),
		Line::from(""),
		Line::from(Span::styled("  exiting...".to_string(), theme.hints_bar)),
	];
	frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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

fn kwic_hints() -> Vec<KeyHint> {
	vec![
		KeyHint::new("?", "help"),
		KeyHint::new("q", "quit"),
	]
}
