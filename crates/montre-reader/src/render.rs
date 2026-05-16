use anyhow::Result;
use montre_tui_core::key_hint::{draw_hints_bar, KeyHint};
use montre_tui_core::sentence_source::{SentenceSource, SentenceView};
use montre_tui_core::status_bar::{draw_status_bar, StatusBarContent};
use montre_tui_core::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::cursor::Cursor;
use crate::daemon_access::DaemonAccess;

pub enum Mode {
	Normal,
	Help,
	ShuttingDown { reason: String },
}

pub struct ViewState<'a> {
	pub cursor: &'a Cursor,
	pub mode: &'a Mode,
	pub connected: bool,
	pub theme: &'a Theme,
}

pub fn draw(frame: &mut Frame, access: &mut DaemonAccess, view: &ViewState) -> Result<()> {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Min(1),
			Constraint::Length(1),
			Constraint::Length(1),
		])
		.split(frame.area());

	let body_area = layout[0];
	let hints_area = layout[1];
	let status_area = layout[2];

	draw_reader_pane(frame, body_area, access, view)?;
	draw_hints_bar(frame, hints_area, &reader_hints(), view.theme);
	draw_reader_status(frame, status_area, access, view)?;

	match view.mode {
		Mode::Normal => {}
		Mode::Help => draw_help_overlay(frame, frame.area(), view.theme),
		Mode::ShuttingDown { reason } => draw_shutdown_overlay(frame, frame.area(), reason, view.theme),
	}

	Ok(())
}

fn draw_reader_pane(
	frame: &mut Frame,
	area: Rect,
	access: &mut DaemonAccess,
	view: &ViewState,
) -> Result<()> {
	let document_name = access
		.document(view.cursor.document_index)
		.map(|summary| summary.name.clone())
		.unwrap_or_default();

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(view.theme.pane_border_active.border_type)
		.border_style(view.theme.pane_border_active.style)
		.title(Span::styled(
			format!(" {} ", document_name),
			view.theme.pane_title,
		));

	let inner = block.inner(area);
	frame.render_widget(block, area);

	let Some(current_summary) = access.document(view.cursor.document_index) else {
		return Ok(());
	};
	let document_index = current_summary.index;
	let sentence_count = current_summary.sentence_count;
	if sentence_count == 0 {
		return Ok(());
	}

	let cursor_offset = view.cursor.sentence_within_document as usize;
	let inner_height = inner.height as usize;
	if inner_height == 0 {
		return Ok(());
	}
	let top_margin = (inner_height / 4).max(2);
	let scroll = cursor_offset.saturating_sub(top_margin);

	let visible_start = scroll as u32;
	let visible_end = (scroll + inner_height).min(sentence_count as usize) as u32;

	let sentences = access.sentences(document_index, visible_start, visible_end)?;

	let cursor_sentence = view.cursor.sentence_within_document;
	let lines: Vec<Line> = sentences
		.iter()
		.map(|sentence| render_sentence_line(sentence, sentence.sentence_within_document == cursor_sentence, view.theme))
		.collect();

	let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
	frame.render_widget(paragraph, inner);
	Ok(())
}

fn render_sentence_line(sentence: &SentenceView, is_cursor: bool, theme: &Theme) -> Line<'static> {
	let (marker, body_style) = if is_cursor {
		(Span::styled("▌ ", theme.cursor_marker), theme.cursor_sentence)
	} else {
		(Span::raw("  "), theme.text_default)
	};
	Line::from(vec![marker, Span::styled(sentence.surface.clone(), body_style)])
}

fn draw_reader_status(
	frame: &mut Frame,
	area: Rect,
	access: &DaemonAccess,
	view: &ViewState,
) -> Result<()> {
	let summary = access.document(view.cursor.document_index);
	let component = summary
		.filter(|_| access.is_multi_component())
		.map(|summary| summary.component.clone());
	let document_name = summary.map(|summary| summary.name.clone()).unwrap_or_default();

	let sentence_id = access
		.document(view.cursor.document_index)
		.map(|summary| {
			let _ = summary;
			view.cursor.sentence_within_document
		});

	let selection = match sentence_id {
		Some(sentence) => format!("{} · sent {}", document_name, sentence),
		None => "—".to_string(),
	};

	let component_ref = component.as_deref();
	let selection_ref = Some(selection.as_str());

	let content = StatusBarContent {
		corpus_name: access.corpus_info.name.as_str(),
		component: component_ref,
		selection: selection_ref,
		anchor_info: None,
		connected: view.connected,
	};
	draw_status_bar(frame, area, &content, view.theme);
	Ok(())
}

fn draw_help_overlay(frame: &mut Frame, area: Rect, theme: &Theme) {
	let overlay_area = centered_rect(60, 70, area);
	frame.render_widget(Clear, overlay_area);

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(theme.overlay_border.border_type)
		.border_style(theme.overlay_border.style)
		.title(Span::styled(" reader help ", theme.overlay_title));
	let inner = block.inner(overlay_area);
	frame.render_widget(block, overlay_area);

	let entries = [
		("↑ ↓  j k", "previous / next sentence"),
		("PgUp PgDn", "screen step"),
		("J K", "previous / next document"),
		("[ ]", "previous / next component"),
		("Home  g", "start of document"),
		("End   G", "end of document"),
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

	let paragraph = Paragraph::new(Text::from(lines));
	frame.render_widget(paragraph, inner);
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
		Line::from(Span::styled(
			format!("  reason: {}", reason),
			theme.error,
		)),
		Line::from(""),
		Line::from(Span::styled(
			"  exiting...".to_string(),
			theme.hints_bar,
		)),
	];
	let paragraph = Paragraph::new(Text::from(lines));
	frame.render_widget(paragraph, inner);
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

pub fn reader_hints() -> Vec<KeyHint> {
	vec![
		KeyHint::new("↑↓ jk", "sentence"),
		KeyHint::new("PgUp/PgDn", "screen"),
		KeyHint::new("J/K", "document"),
		KeyHint::new("[/]", "component"),
		KeyHint::new("Home/g End/G", "doc bounds"),
		KeyHint::new("?", "help"),
		KeyHint::new("q", "quit"),
	]
}
