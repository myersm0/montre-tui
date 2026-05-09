pub struct KeyHint {
	pub key: String,
	pub label: String,
	pub priority: u8,
}

pub fn global_hints() -> Vec<KeyHint> {
	vec![KeyHint {
		key: "q".to_string(),
		label: "quit".to_string(),
		priority: 0,
	}]
}
