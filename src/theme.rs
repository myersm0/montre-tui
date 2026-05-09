use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

pub struct Theme {
	pub pane_border_active: BorderStyle,
	pub pane_border_inactive: BorderStyle,
	pub pane_title: Style,
	pub status_bar: Style,
	pub query_prompt: Style,
	pub query_input: Style,
	pub hints_bar: Style,
	pub hints_key: Style,
	pub hints_separator: Style,
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
			status_bar: Style::default().fg(dim),
			query_prompt: Style::default().fg(accent).add_modifier(Modifier::BOLD),
			query_input: Style::default().fg(foreground),
			hints_bar: Style::default().fg(dim),
			hints_key: Style::default().fg(key_color).add_modifier(Modifier::BOLD),
			hints_separator: Style::default().fg(muted),
			error: Style::default().fg(error_color).add_modifier(Modifier::BOLD),
		}
	}
}
