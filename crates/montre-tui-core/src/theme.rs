use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

pub struct Theme {
	pub pane_border_active: BorderStyle,
	pub pane_border_inactive: BorderStyle,
	pub pane_title: Style,
	pub text_default: Style,
	pub cursor_sentence: Style,
	pub cursor_marker: Style,
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

impl Theme {
	pub fn default_dark() -> Self {
		let accent = Color::Rgb(125, 207, 255);
		let muted = Color::Rgb(86, 95, 137);
		let dim = Color::Rgb(154, 165, 206);
		let foreground = Color::Rgb(192, 202, 245);
		let key_color = Color::Rgb(187, 154, 247);
		let error_color = Color::Rgb(247, 118, 142);
		let cursor_bg = Color::Rgb(40, 50, 78);
		let connected = Color::Rgb(158, 206, 106);
		let disconnected = Color::Rgb(247, 118, 142);

		Self {
			pane_border_active: BorderStyle {
				style: Style::default().fg(accent),
				border_type: BorderType::Double,
			},
			pane_border_inactive: BorderStyle {
				style: Style::default().fg(muted),
				border_type: BorderType::Plain,
			},
			pane_title: Style::default().fg(foreground).add_modifier(Modifier::BOLD),
			text_default: Style::default(),
			cursor_sentence: Style::default().bg(cursor_bg),
			cursor_marker: Style::default()
				.fg(accent)
				.bg(cursor_bg)
				.add_modifier(Modifier::BOLD),
			status_bar: Style::default().fg(dim),
			status_dot_connected: Style::default().fg(connected),
			status_dot_disconnected: Style::default().fg(disconnected),
			hints_bar: Style::default().fg(dim),
			hints_key: Style::default().fg(key_color).add_modifier(Modifier::BOLD),
			hints_separator: Style::default().fg(muted),
			overlay_background: Style::default(),
			overlay_border: BorderStyle {
				style: Style::default().fg(accent),
				border_type: BorderType::Double,
			},
			overlay_title: Style::default().fg(foreground).add_modifier(Modifier::BOLD),
			error: Style::default().fg(error_color).add_modifier(Modifier::BOLD),
		}
	}
}
