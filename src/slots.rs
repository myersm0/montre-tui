use crate::cursor::Cursor;

pub enum SlotContent {
	Empty,
	Reader(ReaderState),
}

pub struct ReaderState {
	pub cursor: Cursor,
	pub scroll_offset: usize,
	pub annotation_overlay: bool,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FocusTarget {
	TopSlot(usize),
	BottomSlot,
}

pub struct PaneLayout {
	pub top_slots: Vec<SlotContent>,
	pub bottom_slot: Option<SlotContent>,
}

impl ReaderState {
	pub fn new() -> Self {
		Self {
			cursor: Cursor {
				document_index: 0,
				sentence_index: 0,
			},
			scroll_offset: 0,
			annotation_overlay: false,
		}
	}
}

impl PaneLayout {
	pub fn new() -> Self {
		Self {
			top_slots: vec![SlotContent::Reader(ReaderState::new())],
			bottom_slot: None,
		}
	}

	pub fn focused_reader_mut(&mut self, focus: FocusTarget) -> Option<&mut ReaderState> {
		match focus {
			FocusTarget::TopSlot(index) => match self.top_slots.get_mut(index)? {
				SlotContent::Reader(state) => Some(state),
				_ => None,
			},
			FocusTarget::BottomSlot => match self.bottom_slot.as_mut()? {
				SlotContent::Reader(state) => Some(state),
				_ => None,
			},
		}
	}

	pub fn focused_slot(&self, focus: FocusTarget) -> Option<&SlotContent> {
		match focus {
			FocusTarget::TopSlot(index) => self.top_slots.get(index),
			FocusTarget::BottomSlot => self.bottom_slot.as_ref(),
		}
	}
}
