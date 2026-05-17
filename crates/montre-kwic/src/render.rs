use anyhow::Result;
use montre_tui_core::key_hint::{draw_hints_bar, KeyHint};
use montre_tui_core::status_bar::{draw_status_bar, StatusBarContent};
use montre_tui_core::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::daemon_access::DaemonAccess;
use crate::page::{HitRow, HitsPage};
use crate::query_bar::QueryBar;

pub enum Mode {
	Normal,
	Edit,
	Help,
	ShuttingDown { reason: String },
}

pub struct ViewState<'a> {
	pub mode: &'a Mode,
	pub query_bar: &'a QueryBar,
	pub page: Option<&'a HitsPage>,
	pub error: Option<&'a str>,
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

	draw_query_bar(frame, query_bar_area, view);
	draw_kwic_pane(frame, body_area, access, view);
	draw_hints_bar(frame, hints_area, &kwic_hints(view), view.theme);
	draw_kwic_status(frame, status_area, access, view);

	match view.mode {
		Mode::Normal | Mode::Edit => {}
		Mode::Help => draw_help_overlay(frame, frame.area(), view.theme),
		Mode::ShuttingDown { reason } => draw_shutdown_overlay(frame, frame.area(), reason, view.theme),
	}

	Ok(())
}

fn draw_query_bar(frame: &mut Frame, area: Rect, view: &ViewState) {
	let prompt_style = match view.mode {
		Mode::Edit => view.theme.cursor_marker,
		_ => view.theme.hints_key,
	};
	let mut spans = vec![Span::styled(" » ", prompt_style)];

	if matches!(view.mode, Mode::Edit) {
		let text = &view.query_bar.text;
		let cursor = view.query_bar.cursor_byte.min(text.len());
		let (before_cursor, at_and_after) = text.split_at(cursor);
		spans.push(Span::styled(before_cursor.to_string(), view.theme.text_default));
		if let Some(next_char) = at_and_after.chars().next() {
			let next_char_len = next_char.len_utf8();
			spans.push(Span::styled(next_char.to_string(), view.theme.cursor_sentence));
			spans.push(Span::styled(
				at_and_after[next_char_len..].to_string(),
				view.theme.text_default,
			));
		} else {
			spans.push(Span::styled(" ", view.theme.cursor_sentence));
		}
	} else {
		spans.push(Span::styled(view.query_bar.text.clone(), view.theme.hints_bar));
	}

	frame.render_widget(Line::from(spans), area);
}

fn draw_kwic_pane(frame: &mut Frame, area: Rect, access: &DaemonAccess, view: &ViewState) {
	let title = match view.page {
		Some(page) => format!(" KWIC ({} hits) ", page.hit_count),
		None => " KWIC ".to_string(),
	};

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(view.theme.pane_border_active.border_type)
		.border_style(view.theme.pane_border_active.style)
		.title(Span::styled(title, view.theme.pane_title));
	let inner = block.inner(area);
	frame.render_widget(block, area);

	if let Some(error) = view.error {
		let paragraph = Paragraph::new(Text::from(Line::from(Span::styled(
			error.to_string(),
			view.theme.error,
		))));
		frame.render_widget(paragraph, inner);
		return;
	}

	let Some(page) = view.page else {
		let placeholder = Line::from(Span::styled("No query yet.", view.theme.hints_bar));
		frame.render_widget(Paragraph::new(Text::from(placeholder)), inner);
		return;
	};

	if page.rows.is_empty() {
		let placeholder = Line::from(Span::styled("No matches.", view.theme.hints_bar));
		frame.render_widget(Paragraph::new(Text::from(placeholder)), inner);
		return;
	}

	let inner_height = inner.height as usize;
	if inner_height == 0 {
		return;
	}
	let top_margin = (inner_height / 4).max(2);
	let scroll = page.cursor.saturating_sub(top_margin);
	let visible_end = (scroll + inner_height).min(page.rows.len());

	let mut lines: Vec<Line> = page.rows[scroll..visible_end]
		.iter()
		.enumerate()
		.map(|(local_index, row)| {
			let global_index = scroll + local_index;
			let is_cursor = global_index == page.cursor;
			render_hit_row(row, access, is_cursor, view.theme)
		})
		.collect();

	if visible_end >= page.rows.len() && (page.rows.len() as u64) < page.hit_count {
		lines.push(Line::from(Span::styled(
			format!(
				"  (+{} more — paging lands in v1)",
				page.hit_count - page.rows.len() as u64,
			),
			view.theme.hints_bar,
		)));
	}

	frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn render_hit_row(row: &HitRow, access: &DaemonAccess, is_cursor: bool, theme: &Theme) -> Line<'static> {
	let document_name = access
		.document(row.document_index)
		.map(|summary| summary.name.clone())
		.unwrap_or_default();
	let doc_col_width: usize = 18;
	let context_col_width: usize = 30;
	let doc_field = pad_or_truncate_right(&document_name, doc_col_width);
	let sent_field = format!("sent {:>5}", row.sentence_index);
	let left_field = pad_or_truncate_left(row.left_text.trim(), context_col_width);
	let right_field = pad_or_truncate_right(row.right_text.trim(), context_col_width);
	let row_style = if is_cursor {
		theme.cursor_sentence
	} else {
		theme.text_default
	};
	Line::from(vec![
		Span::styled(doc_field, theme.hints_bar),
		Span::raw(" "),
		Span::styled(sent_field, theme.hints_bar),
		Span::raw("  "),
		Span::styled(left_field, theme.hints_bar),
		Span::raw(" "),
		Span::styled(
			row.match_text.clone(),
			theme.text_default.add_modifier(Modifier::BOLD),
		),
		Span::raw(" "),
		Span::styled(right_field, theme.hints_bar),
	])
	.style(row_style)
}

fn pad_or_truncate_right(text: &str, width: usize) -> String {
	let char_count = text.chars().count();
	if char_count == width {
		text.to_string()
	} else if char_count < width {
		format!("{}{}", text, " ".repeat(width - char_count))
	} else {
		let mut result: String = text.chars().take(width.saturating_sub(1)).collect();
		result.push('…');
		result
	}
}

fn pad_or_truncate_left(text: &str, width: usize) -> String {
	let char_count = text.chars().count();
	if char_count == width {
		text.to_string()
	} else if char_count < width {
		format!("{}{}", " ".repeat(width - char_count), text)
	} else {
		let skip = char_count - width.saturating_sub(1);
		let kept: String = text.chars().skip(skip).collect();
		format!("…{}", kept)
	}
}

fn draw_kwic_status(frame: &mut Frame, area: Rect, access: &DaemonAccess, view: &ViewState) {
	let selection_string;
	let selection = match view.page {
		Some(page) if !page.rows.is_empty() => {
			selection_string = format!("hit {} of {}", page.cursor + 1, page.hit_count);
			Some(selection_string.as_str())
		}
		_ => None,
	};
	let content = StatusBarContent {
		corpus_name: access.corpus_info.name.as_str(),
		component: None,
		selection,
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
		("i", "enter query"),
		("↑ ↓  j k", "previous / next hit"),
		("PgUp PgDn", "screen step"),
		("Home  g", "first hit"),
		("End   G", "last hit"),
		("Enter", "republish current hit"),
		("", ""),
		("Edit mode:", ""),
		("Enter", "run query"),
		("Esc", "cancel edit"),
		("← →", "cursor"),
		("Home End", "start / end of input"),
		("", ""),
		("?", "toggle help"),
		("q  Esc", "quit (Normal)"),
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

fn kwic_hints(view: &ViewState) -> Vec<KeyHint> {
	match view.mode {
		Mode::Edit => vec![
			KeyHint::new("Enter", "run"),
			KeyHint::new("Esc", "cancel"),
		],
		Mode::Normal if view.page.is_some() => vec![
			KeyHint::new("i", "query"),
			KeyHint::new("jk", "hit"),
			KeyHint::new("PgUp/PgDn", "screen"),
			KeyHint::new("?", "help"),
			KeyHint::new("q", "quit"),
		],
		Mode::Normal => vec![
			KeyHint::new("i", "query"),
			KeyHint::new("?", "help"),
			KeyHint::new("q", "quit"),
		],
		_ => vec![],
	}
}
