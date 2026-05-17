use ratatui::style::{Modifier, Style};
use ratatui::widgets::BorderType;

use crate::palette::Palette;

pub struct Theme {
	pub page_background: Style,
	pub pane_border_active: BorderStyle,
	pub pane_border_inactive: BorderStyle,
	pub pane_title: Style,
	pub text_default: Style,
	pub text_subtle: Style,
	pub selected_row: Style,
	pub selected_text_default: Style,
	pub selected_text_subtle: Style,
	pub selected_kwic_match: Style,
	pub selected_cursor_marker: Style,
	pub input_cursor: Style,
	pub cursor_marker: Style,
	pub kwic_match: Style,
	pub status_bar: Style,
	pub status_dot_connected: Style,
	pub status_dot_disconnected: Style,
	pub hints_bar: Style,
	pub hints_key: Style,
	pub hints_separator: Style,
	pub overlay_background: Style,
	pub overlay_border: BorderStyle,
	pub overlay_title: Style,
	pub error: Style,
}

pub struct BorderStyle {
	pub style: Style,
	pub border_type: BorderType,
}

pub struct RowStyles {
	pub background: Style,
	pub text_default: Style,
	pub text_subtle: Style,
	pub kwic_match: Style,
	pub cursor_marker: Style,
}

impl Theme {
	pub fn from_palette(palette: &Palette) -> Self {
		Self {
			page_background: Style::default().bg(palette.page),
			pane_border_active: BorderStyle {
				style: Style::default().fg(palette.border_strong),
				border_type: BorderType::Double,
			},
			pane_border_inactive: BorderStyle {
				style: Style::default().fg(palette.border_subtle),
				border_type: BorderType::Plain,
			},
			pane_title: Style::default()
				.fg(palette.text_strong)
				.add_modifier(Modifier::BOLD),
			text_default: Style::default().fg(palette.text_body),
			text_subtle: Style::default().fg(palette.text_muted),
			selected_row: Style::default().bg(palette.highlight_page),
			selected_text_default: Style::default()
				.fg(palette.highlight_body)
				.bg(palette.highlight_page),
			selected_text_subtle: Style::default()
				.fg(palette.highlight_muted)
				.bg(palette.highlight_page),
			selected_kwic_match: Style::default()
				.fg(palette.highlight_strong)
				.bg(palette.highlight_page)
				.add_modifier(Modifier::BOLD),
			selected_cursor_marker: Style::default()
				.fg(palette.highlight_brick)
				.bg(palette.highlight_page)
				.add_modifier(Modifier::BOLD),
			input_cursor: Style::default()
				.fg(palette.text_strong)
				.bg(palette.honey),
			cursor_marker: Style::default()
				.fg(palette.brick)
				.add_modifier(Modifier::BOLD),
			kwic_match: Style::default()
				.fg(palette.text_strong)
				.add_modifier(Modifier::BOLD),
			status_bar: Style::default().fg(palette.text_muted),
			status_dot_connected: Style::default().fg(palette.verdigris),
			status_dot_disconnected: Style::default().fg(palette.brick),
			hints_bar: Style::default().fg(palette.text_muted),
			hints_key: Style::default()
				.fg(palette.text_strong)
				.add_modifier(Modifier::BOLD),
			hints_separator: Style::default().fg(palette.border_subtle),
			overlay_background: Style::default().bg(palette.recessed),
			overlay_border: BorderStyle {
				style: Style::default().fg(palette.border_strong),
				border_type: BorderType::Double,
			},
			overlay_title: Style::default()
				.fg(palette.text_strong)
				.add_modifier(Modifier::BOLD),
			error: Style::default()
				.fg(palette.brick)
				.add_modifier(Modifier::BOLD),
		}
	}

	pub fn row_styles(&self, selected: bool) -> RowStyles {
		if selected {
			RowStyles {
				background: self.selected_row,
				text_default: self.selected_text_default,
				text_subtle: self.selected_text_subtle,
				kwic_match: self.selected_kwic_match,
				cursor_marker: self.selected_cursor_marker,
			}
		} else {
			RowStyles {
				background: Style::default(),
				text_default: self.text_default,
				text_subtle: self.text_subtle,
				kwic_match: self.kwic_match,
				cursor_marker: self.cursor_marker,
			}
		}
	}
}
