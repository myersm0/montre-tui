use ratatui::style::Color;

pub struct Palette {
	pub page: Color,
	pub elevated: Color,
	pub recessed: Color,
	pub border_subtle: Color,
	pub border_strong: Color,
	pub text_muted: Color,
	pub text_body: Color,
	pub text_strong: Color,
	pub brick: Color,
	pub honey: Color,
	pub verdigris: Color,
}

impl Palette {
	pub fn grundtvig_dark() -> Self {
		Self {
			page: Color::Rgb(0x1d, 0x0d, 0x03),
			elevated: Color::Rgb(0x2f, 0x19, 0x08),
			recessed: Color::Rgb(0x15, 0x08, 0x02),
			border_subtle: Color::Rgb(0x43, 0x26, 0x10),
			border_strong: Color::Rgb(0x6c, 0x41, 0x1a),
			text_muted: Color::Rgb(0xa8, 0x7b, 0x57),
			text_body: Color::Rgb(0xe3, 0xc9, 0xa6),
			text_strong: Color::Rgb(0xf9, 0xe8, 0xd2),
			brick: Color::Rgb(0xe9, 0x6e, 0x50),
			honey: Color::Rgb(0xe3, 0xb4, 0x52),
			verdigris: Color::Rgb(0x6d, 0xae, 0x97),
		}
	}
}
