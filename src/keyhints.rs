use crate::slots::SlotContent;

pub struct KeyHint {
	pub key: String,
	pub label: String,
	pub priority: u8,
}

impl KeyHint {
	fn new(key: &str, label: &str, priority: u8) -> Self {
		Self {
			key: key.to_string(),
			label: label.to_string(),
			priority,
		}
	}
}

pub fn global_hints() -> Vec<KeyHint> {
	vec![KeyHint::new("q", "quit", 0)]
}

pub fn hints_for_focused_slot(content: Option<&SlotContent>) -> Vec<KeyHint> {
	match content {
		Some(SlotContent::Reader(_)) => reader_hints(),
		_ => Vec::new(),
	}
}

fn reader_hints() -> Vec<KeyHint> {
	vec![
		KeyHint::new("↑↓", "sentence", 100),
		KeyHint::new("PgUp/PgDn", "screen", 90),
		KeyHint::new("J/K", "document", 80),
		KeyHint::new("[/]", "component", 70),
		KeyHint::new("Home/End", "doc bounds", 60),
	]
}
