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

	pub fn open_slot_after(&mut self, focus: FocusTarget) -> FocusTarget {
		if self.top_slots.len() >= 3 {
			return focus;
		}
		let insert_at = match focus {
			FocusTarget::TopSlot(index) => index + 1,
			FocusTarget::BottomSlot => self.top_slots.len(),
		};
		self.top_slots.insert(insert_at, SlotContent::Empty);
		FocusTarget::TopSlot(insert_at)
	}

	pub fn close_slot(&mut self, focus: FocusTarget) -> FocusTarget {
		match focus {
			FocusTarget::TopSlot(index) => {
				if self.top_slots.len() <= 1 {
					return focus;
				}
				self.top_slots.remove(index);
				let next_index = index.saturating_sub(1).min(self.top_slots.len() - 1);
				FocusTarget::TopSlot(next_index)
			}
			FocusTarget::BottomSlot => focus,
		}
	}

	pub fn cycle_focus_forward(&self, focus: FocusTarget) -> FocusTarget {
		match focus {
			FocusTarget::TopSlot(index) => {
				let next = (index + 1) % self.top_slots.len();
				FocusTarget::TopSlot(next)
			}
			FocusTarget::BottomSlot => FocusTarget::TopSlot(0),
		}
	}

	pub fn cycle_focus_backward(&self, focus: FocusTarget) -> FocusTarget {
		match focus {
			FocusTarget::TopSlot(index) => {
				let previous = if index == 0 {
					self.top_slots.len() - 1
				} else {
					index - 1
				};
				FocusTarget::TopSlot(previous)
			}
			FocusTarget::BottomSlot => FocusTarget::TopSlot(self.top_slots.len() - 1),
		}
	}

	pub fn set_focused_content(&mut self, focus: FocusTarget, content: SlotContent) {
		match focus {
			FocusTarget::TopSlot(index) => {
				if let Some(slot) = self.top_slots.get_mut(index) {
					*slot = content;
				}
			}
			FocusTarget::BottomSlot => {
				if let Some(slot) = self.bottom_slot.as_mut() {
					*slot = content;
				}
			}
		}
	}
}
