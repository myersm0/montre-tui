use montre_query::executor::Results;

use crate::cursor::Cursor;

pub enum SlotContent {
	Empty,
	Reader(ReaderState),
	Kwic(KwicState),
}

pub struct ReaderState {
	pub cursor: Cursor,
	pub scroll_offset: usize,
	pub annotation_overlay: bool,
}

pub struct KwicState {
	pub results: Option<Results>,
	pub error: Option<String>,
	pub selected: usize,
	pub scroll_offset: usize,
	pub window_tokens: usize,
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

impl KwicState {
	pub fn new() -> Self {
		Self {
			results: None,
			error: None,
			selected: 0,
			scroll_offset: 0,
			window_tokens: 8,
		}
	}

	pub fn select_next(&mut self) {
		if let Some(results) = &self.results {
			let length = results.hits().len();
			if length > 0 && self.selected + 1 < length {
				self.selected += 1;
			}
		}
	}

	pub fn select_previous(&mut self) {
		self.selected = self.selected.saturating_sub(1);
	}
}

impl PaneLayout {
	pub fn new() -> Self {
		Self {
			top_slots: vec![SlotContent::Reader(ReaderState::new())],
			bottom_slot: Some(SlotContent::Kwic(KwicState::new())),
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

	pub fn focused_kwic_mut(&mut self, focus: FocusTarget) -> Option<&mut KwicState> {
		match focus {
			FocusTarget::TopSlot(index) => match self.top_slots.get_mut(index)? {
				SlotContent::Kwic(state) => Some(state),
				_ => None,
			},
			FocusTarget::BottomSlot => match self.bottom_slot.as_mut()? {
				SlotContent::Kwic(state) => Some(state),
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

	pub fn ensure_kwic_slot(&mut self) -> &mut KwicState {
		let already_exists = self
			.top_slots
			.iter()
			.any(|slot| matches!(slot, SlotContent::Kwic(_)))
			|| matches!(self.bottom_slot, Some(SlotContent::Kwic(_)));
		if !already_exists {
			self.bottom_slot = Some(SlotContent::Kwic(KwicState::new()));
		}
		self.first_kwic_mut().expect("ensured")
	}

	pub fn ensure_reader_slot(&mut self) -> Option<FocusTarget> {
		if let Some(index) = self
			.top_slots
			.iter()
			.position(|slot| matches!(slot, SlotContent::Reader(_)))
		{
			return Some(FocusTarget::TopSlot(index));
		}
		if matches!(self.bottom_slot, Some(SlotContent::Reader(_))) {
			return Some(FocusTarget::BottomSlot);
		}
		if self.top_slots.len() < 3 {
			self.top_slots
				.push(SlotContent::Reader(ReaderState::new()));
			Some(FocusTarget::TopSlot(self.top_slots.len() - 1))
		} else {
			None
		}
	}

	pub fn first_kwic_mut(&mut self) -> Option<&mut KwicState> {
		for slot in self.top_slots.iter_mut() {
			if let SlotContent::Kwic(state) = slot {
				return Some(state);
			}
		}
		if let Some(SlotContent::Kwic(state)) = self.bottom_slot.as_mut() {
			return Some(state);
		}
		None
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
				let next = index + 1;
				if next < self.top_slots.len() {
					FocusTarget::TopSlot(next)
				} else if self.bottom_slot.is_some() {
					FocusTarget::BottomSlot
				} else {
					FocusTarget::TopSlot(0)
				}
			}
			FocusTarget::BottomSlot => FocusTarget::TopSlot(0),
		}
	}

	pub fn cycle_focus_backward(&self, focus: FocusTarget) -> FocusTarget {
		match focus {
			FocusTarget::TopSlot(index) => {
				if index > 0 {
					FocusTarget::TopSlot(index - 1)
				} else if self.bottom_slot.is_some() {
					FocusTarget::BottomSlot
				} else {
					FocusTarget::TopSlot(self.top_slots.len() - 1)
				}
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
