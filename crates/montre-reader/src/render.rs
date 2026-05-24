use anyhow::Result;
use montre_tui_core::key_hint::{draw_hints_bar, KeyHint};
use montre_tui_core::overlay;
use montre_tui_core::sentence_source::{SentenceSource, SentenceView};
use montre_tui_core::status_bar::{draw_status_bar, StatusBarContent};
use montre_tui_core::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
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
	frame.render_widget(Block::default().style(view.theme.page_background), frame.area());
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
		Mode::Help => overlay::draw_help(frame, frame.area(), view.theme, "reader help", &reader_help_entries()),
		Mode::ShuttingDown { reason } => overlay::draw_shutdown(frame, frame.area(), reason, view.theme),
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
	let styles = theme.row_styles(is_cursor);
	let marker = if is_cursor {
		Span::styled("▌ ", styles.cursor_marker)
	} else {
		Span::raw("  ")
	};
	Line::from(vec![marker, Span::styled(sentence.surface.clone(), styles.text_default)])
		.style(styles.background)
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
		coupler_info: None,
		connected: view.connected,
	};
	draw_status_bar(frame, area, &content, view.theme);
	Ok(())
}

fn reader_help_entries() -> &'static [(&'static str, &'static str)] {
	&[
		("↑ ↓  j k", "previous / next sentence"),
		("PgUp PgDn", "screen step"),
		("J K", "previous / next document"),
		("[ ]", "previous / next component"),
		("Home  g", "start of document"),
		("End   G", "end of document"),
		("?", "toggle help"),
		("q  Esc", "quit"),
	]
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
