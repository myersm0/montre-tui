use crate::app::Mode;
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

pub fn hints_for(mode: Mode, focused: Option<&SlotContent>) -> Vec<KeyHint> {
	match mode {
		Mode::QueryEntry => query_entry_hints(),
		Mode::Normal => {
			let mut hints = hints_for_focused_slot(focused);
			hints.extend(global_hints());
			hints.sort_by(|a, b| b.priority.cmp(&a.priority));
			hints
		}
	}
}

fn global_hints() -> Vec<KeyHint> {
	vec![
		KeyHint::new(":", "query", 3),
		KeyHint::new("Tab", "focus", 2),
		KeyHint::new("+/-", "open/close slot", 1),
		KeyHint::new("q", "quit", 0),
	]
}

fn hints_for_focused_slot(content: Option<&SlotContent>) -> Vec<KeyHint> {
	match content {
		Some(SlotContent::Reader(_)) => reader_hints(),
		Some(SlotContent::Empty) => empty_slot_hints(),
		Some(SlotContent::Kwic(_)) => kwic_hints(),
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

fn empty_slot_hints() -> Vec<KeyHint> {
	vec![KeyHint::new("r", "reader", 100)]
}

fn kwic_hints() -> Vec<KeyHint> {
	vec![KeyHint::new("↑↓", "select", 100)]
}

fn query_entry_hints() -> Vec<KeyHint> {
	vec![
		KeyHint::new("Enter", "execute", 100),
		KeyHint::new("Esc", "cancel", 90),
	]
}
