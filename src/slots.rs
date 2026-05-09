pub enum SlotContent {
	Empty,
}

pub struct PaneLayout {
	pub top_slots: Vec<SlotContent>,
	pub bottom_slot: Option<SlotContent>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FocusTarget {
	TopSlot(usize),
	BottomSlot,
}

impl PaneLayout {
	pub fn new() -> Self {
		Self {
			top_slots: vec![SlotContent::Empty],
			bottom_slot: None,
		}
	}
}
