use anyhow::Result;
use montre_tui_core::key_hint::{draw_hints_bar, KeyHint};
use montre_tui_core::overlay;
use montre_tui_core::protocol::Span;
use montre_tui_core::status_bar::{draw_status_bar, StatusBarContent};
use montre_tui_core::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span as TextSpan, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::daemon_access::{DaemonAccess, TokenWindow};

pub enum Mode {
	Normal,
	Help,
	ShuttingDown { reason: String },
}

pub struct ViewState<'a> {
	pub cursor_position: u64,
	pub window: &'a TokenWindow,
	pub highlight: Option<&'a Span>,
	pub mode: &'a Mode,
	pub connected: bool,
	pub theme: &'a Theme,
}

pub struct LayoutToken {
	pub position: u64,
	pub byte_start: usize,
	pub byte_end: usize,
	pub column: usize,
}

pub struct LayoutRow {
	pub byte_start: usize,
	pub byte_end: usize,
	pub tokens: Vec<LayoutToken>,
}

pub struct WrapLayout {
	pub rows: Vec<LayoutRow>,
}

impl WrapLayout {
	pub fn row_of_position(&self, position: u64) -> Option<usize> {
		self.rows
			.iter()
			.position(|row| row.tokens.iter().any(|token| token.position == position))
	}

	pub fn column_of_position(&self, position: u64) -> Option<usize> {
		for row in &self.rows {
			for token in &row.tokens {
				if token.position == position {
					return Some(token.column);
				}
			}
		}
		None
	}

	pub fn position_by_row_delta(&self, position: u64, delta: isize) -> Option<u64> {
		let row = self.row_of_position(position)?;
		let column = self.column_of_position(position)?;
		let step = if delta < 0 { -1 } else { 1 };
		let mut target = row as isize + delta;
		loop {
			if target < 0 || target as usize >= self.rows.len() {
				return None;
			}
			let candidate = &self.rows[target as usize];
			if !candidate.tokens.is_empty() {
				return candidate
					.tokens
					.iter()
					.min_by_key(|token| token.column.abs_diff(column))
					.map(|token| token.position);
			}
			target += step;
		}
	}
}

struct Segment {
	start: usize,
	end: usize,
	width: usize,
}

pub fn build_layout(window: &TokenWindow, width: usize) -> WrapLayout {
	let width = width.max(1);
	let surface = &window.surface;
	let total = surface.len();
	let mut rows: Vec<LayoutRow> = Vec::new();
	let mut line_start = 0;
	loop {
		let newline = surface[line_start..].find('\n').map(|offset| line_start + offset);
		let line_end = newline.unwrap_or(total);
		let content_end = if line_end > line_start && surface.as_bytes()[line_end - 1] == b'\r' {
			line_end - 1
		} else {
			line_end
		};
		wrap_hard_line(window, line_start, content_end, width, &mut rows);
		match newline {
			Some(position) => line_start = position + 1,
			None => break,
		}
	}
	WrapLayout { rows }
}

fn wrap_hard_line(window: &TokenWindow, line_start: usize, line_end: usize, width: usize, rows: &mut Vec<LayoutRow>) {
	let segments = split_segments(&window.surface, line_start, line_end);
	let Some(first) = segments.first() else {
		rows.push(LayoutRow { byte_start: line_start, byte_end: line_start, tokens: Vec::new() });
		return;
	};
	let mut row_start = first.start;
	let mut row_end = first.end;
	let mut row_width = first.width;
	for segment in &segments[1..] {
		if row_width + 1 + segment.width > width {
			rows.push(finish_row(window, row_start, row_end));
			row_start = segment.start;
			row_end = segment.end;
			row_width = segment.width;
		} else {
			row_width += 1 + segment.width;
			row_end = segment.end;
		}
	}
	rows.push(finish_row(window, row_start, row_end));
}

fn split_segments(surface: &str, range_start: usize, range_end: usize) -> Vec<Segment> {
	let bytes = surface.as_bytes();
	let mut segments = Vec::new();
	let mut index = range_start;
	while index < range_end {
		if bytes[index] == b' ' {
			index += 1;
			continue;
		}
		let start = index;
		while index < range_end && bytes[index] != b' ' {
			index += 1;
		}
		segments.push(Segment {
			start,
			end: index,
			width: surface[start..index].width(),
		});
	}
	segments
}

fn finish_row(window: &TokenWindow, byte_start: usize, byte_end: usize) -> LayoutRow {
	let mut tokens = Vec::new();
	for token in &window.tokens {
		if !token.emitted {
			continue;
		}
		let token_start = token.surface_start as usize;
		if token_start >= byte_start && token_start < byte_end {
			tokens.push(LayoutToken {
				position: token.position,
				byte_start: token_start,
				byte_end: token.surface_end as usize,
				column: window.surface[byte_start..token_start].width(),
			});
		}
	}
	LayoutRow { byte_start, byte_end, tokens }
}

pub fn draw(frame: &mut Frame, access: &DaemonAccess, view: &ViewState) -> Result<()> {
	frame.render_widget(Block::default().style(view.theme.page_background), frame.area());
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
		.split(frame.area());

	let body_area = layout[0];
	let hints_area = layout[1];
	let status_area = layout[2];

	draw_reader_pane(frame, body_area, access, view);
	draw_hints_bar(frame, hints_area, &reader_hints(), view.theme);
	draw_reader_status(frame, status_area, access, view);

	match view.mode {
		Mode::Normal => {}
		Mode::Help => overlay::draw_help(frame, frame.area(), view.theme, "reader help", &reader_help_entries()),
		Mode::ShuttingDown { reason } => overlay::draw_shutdown(frame, frame.area(), reason, view.theme),
	}
	Ok(())
}

fn draw_reader_pane(frame: &mut Frame, area: Rect, access: &DaemonAccess, view: &ViewState) {
	let document_name = access
		.document_of_position(view.cursor_position)
		.and_then(|index| access.document(index))
		.map(|summary| summary.name.clone())
		.unwrap_or_default();

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(view.theme.pane_border_active.border_type)
		.border_style(view.theme.pane_border_active.style)
		.title(TextSpan::styled(format!(" {} ", document_name), view.theme.pane_title));

	let inner = block.inner(area);
	frame.render_widget(block, area);
	if inner.width == 0 || inner.height == 0 {
		return;
	}

	let layout = build_layout(view.window, inner.width as usize);
	if layout.rows.is_empty() {
		return;
	}

	let cursor_row = layout.row_of_position(view.cursor_position).unwrap_or(0);
	let inner_height = inner.height as usize;
	let top_margin = (inner_height / 4).max(2);
	let scroll = cursor_row.saturating_sub(top_margin);
	let visible_end = (scroll + inner_height).min(layout.rows.len());

	let lines: Vec<Line> = layout.rows[scroll..visible_end]
		.iter()
		.map(|row| render_row(row, view))
		.collect();

	frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn render_row(row: &LayoutRow, view: &ViewState) -> Line<'static> {
	let surface = &view.window.surface;
	let mut spans: Vec<TextSpan> = Vec::new();
	let mut byte = row.byte_start;
	for token in &row.tokens {
		if token.byte_start > byte {
			spans.push(TextSpan::styled(surface[byte..token.byte_start].to_string(), view.theme.text_default));
		}
		spans.push(TextSpan::styled(
			surface[token.byte_start..token.byte_end].to_string(),
			token_style(token.position, view),
		));
		byte = token.byte_end;
	}
	if byte < row.byte_end {
		spans.push(TextSpan::styled(surface[byte..row.byte_end].to_string(), view.theme.text_default));
	}
	Line::from(spans)
}

fn token_style(position: u64, view: &ViewState) -> Style {
	let highlighted = view
		.highlight
		.map(|span| position >= span.start && position < span.end)
		.unwrap_or(false);
	let mut style = if highlighted { view.theme.kwic_match } else { view.theme.text_default };
	if position == view.cursor_position {
		style = style.add_modifier(Modifier::UNDERLINED);
	}
	style
}

fn draw_reader_status(frame: &mut Frame, area: Rect, access: &DaemonAccess, view: &ViewState) {
	let summary = access.document_of_position(view.cursor_position).and_then(|index| access.document(index));
	let component = summary.filter(|_| access.is_multi_component()).map(|summary| summary.component.clone());
	let selection = match summary {
		Some(summary) => format!("{} · tok {}", summary.name, view.cursor_position),
		None => "—".to_string(),
	};

	let content = StatusBarContent {
		corpus_name: access.corpus_info.name.as_str(),
		component: component.as_deref(),
		selection: Some(selection.as_str()),
		coupler_info: None,
		connected: view.connected,
	};
	draw_status_bar(frame, area, &content, view.theme);
}

fn reader_help_entries() -> &'static [(&'static str, &'static str)] {
	&[
		("← →  h l", "previous / next token"),
		("↑ ↓  j k", "previous / next line"),
		("PgUp PgDn", "page"),
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
		KeyHint::new("←→ hl", "token"),
		KeyHint::new("↑↓ jk", "line"),
		KeyHint::new("PgUp/PgDn", "page"),
		KeyHint::new("J/K", "document"),
		KeyHint::new("[/]", "component"),
		KeyHint::new("?", "help"),
		KeyHint::new("q", "quit"),
	]
}
