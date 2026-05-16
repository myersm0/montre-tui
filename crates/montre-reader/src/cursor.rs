use crate::daemon_access::DaemonAccess;

pub struct Cursor {
	pub document_index: u32,
	pub sentence_within_document: u32,
}

impl Cursor {
	pub fn at_corpus_start() -> Self {
		Self {
			document_index: 0,
			sentence_within_document: 0,
		}
	}

	pub fn jump_to(&mut self, document_index: u32, sentence_within_document: u32) {
		self.document_index = document_index;
		self.sentence_within_document = sentence_within_document;
	}

	pub fn advance_sentence(&mut self, access: &DaemonAccess) {
		let Some(current) = access.document(self.document_index) else {
			return;
		};
		if self.sentence_within_document + 1 < current.sentence_count {
			self.sentence_within_document += 1;
		} else if self.document_index + 1 < access.document_count() {
			self.document_index += 1;
			self.sentence_within_document = 0;
		}
	}

	pub fn retreat_sentence(&mut self, access: &DaemonAccess) {
		if self.sentence_within_document > 0 {
			self.sentence_within_document -= 1;
		} else if self.document_index > 0 {
			self.document_index -= 1;
			let previous = access.document(self.document_index);
			self.sentence_within_document = previous
				.map(|summary| summary.sentence_count.saturating_sub(1))
				.unwrap_or(0);
		}
	}

	pub fn advance_screen(&mut self, access: &DaemonAccess, height: usize) {
		let total = access.total_sentences();
		if total == 0 {
			return;
		}
		let current_global = access.to_global_sentence(self.document_index, self.sentence_within_document);
		let target = (current_global + height as u64).min(total - 1);
		let (document_index, sentence_within_document) = access.from_global_sentence(target);
		self.document_index = document_index;
		self.sentence_within_document = sentence_within_document;
	}

	pub fn retreat_screen(&mut self, access: &DaemonAccess, height: usize) {
		let current_global = access.to_global_sentence(self.document_index, self.sentence_within_document);
		let target = current_global.saturating_sub(height as u64);
		let (document_index, sentence_within_document) = access.from_global_sentence(target);
		self.document_index = document_index;
		self.sentence_within_document = sentence_within_document;
	}

	pub fn advance_document(&mut self, access: &DaemonAccess) {
		if self.document_index + 1 < access.document_count() {
			self.document_index += 1;
			self.sentence_within_document = 0;
		}
	}

	pub fn retreat_document(&mut self, access: &DaemonAccess) {
		if self.document_index > 0 {
			self.document_index -= 1;
			self.sentence_within_document = 0;
		} else {
			let _ = access;
		}
	}

	pub fn advance_component(&mut self, access: &DaemonAccess) {
		if let Some(target) = access.first_document_of_next_component(self.document_index) {
			self.document_index = target;
			self.sentence_within_document = 0;
		}
	}

	pub fn retreat_component(&mut self, access: &DaemonAccess) {
		if let Some(target) = access.first_document_of_previous_component(self.document_index) {
			self.document_index = target;
			self.sentence_within_document = 0;
		}
	}

	pub fn to_document_start(&mut self) {
		self.sentence_within_document = 0;
	}

	pub fn to_document_end(&mut self, access: &DaemonAccess) {
		if let Some(summary) = access.document(self.document_index) {
			self.sentence_within_document = summary.sentence_count.saturating_sub(1);
		}
	}
}
