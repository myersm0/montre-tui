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
	pub highlight_page: Color,
	pub highlight_body: Color,
	pub highlight_strong: Color,
	pub highlight_muted: Color,
	pub highlight_brick: Color,
	pub highlight_honey: Color,
	pub highlight_verdigris: Color,
}

impl Palette {
	pub fn grundtvig_dark() -> Self {
		Self {
			page: Color::from_u32(0x001d0d03),
			elevated: Color::from_u32(0x00e95e50),
			recessed: Color::from_u32(0x00150802),
			border_subtle: Color::from_u32(0x00432610),
			border_strong: Color::from_u32(0x006c411a),
			text_muted: Color::from_u32(0x00d3b08f),
			text_body: Color::from_u32(0x00feebd6),
			text_strong: Color::from_u32(0x00fff7e6),
			brick: Color::from_u32(0x00e96e50),
			honey: Color::from_u32(0x00e3b452),
			verdigris: Color::from_u32(0x006dae97),
			highlight_page: Color::from_u32(0x00fdecd5),
			highlight_body: Color::from_u32(0x00562f12),
			highlight_strong: Color::from_u32(0x002a1404),
			highlight_muted: Color::from_u32(0x008f5c2f),
			highlight_brick: Color::from_u32(0x00ae4024),
			highlight_honey: Color::from_u32(0x00e8b347),
			highlight_verdigris: Color::from_u32(0x0030715d),
		}
	}
}
