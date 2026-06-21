use crate::daemon_access::DaemonAccess;

pub struct Cursor {
	pub position: u64,
}

impl Cursor {
	pub fn at_corpus_start() -> Self {
		Self { position: 0 }
	}

	pub fn set_position(&mut self, position: u64, access: &DaemonAccess) {
		let ceiling = access.token_ceiling();
		self.position = if ceiling == 0 { 0 } else { position.min(ceiling - 1) };
	}

	pub fn advance_document(&mut self, access: &DaemonAccess) {
		let Some(current) = access.document_of_position(self.position) else {
			return;
		};
		if let Some(next) = access.document(current + 1) {
			self.position = next.token_start;
		}
	}

	pub fn retreat_document(&mut self, access: &DaemonAccess) {
		let Some(current) = access.document_of_position(self.position) else {
			return;
		};
		if current > 0 {
			if let Some(previous) = access.document(current - 1) {
				self.position = previous.token_start;
			}
		}
	}

	pub fn advance_component(&mut self, access: &DaemonAccess) {
		let Some(current) = access.document_of_position(self.position) else {
			return;
		};
		if let Some(target) = access.first_document_of_next_component(current) {
			if let Some(document) = access.document(target) {
				self.position = document.token_start;
			}
		}
	}

	pub fn retreat_component(&mut self, access: &DaemonAccess) {
		let Some(current) = access.document_of_position(self.position) else {
			return;
		};
		if let Some(target) = access.first_document_of_previous_component(current) {
			if let Some(document) = access.document(target) {
				self.position = document.token_start;
			}
		}
	}

	pub fn to_document_start(&mut self, access: &DaemonAccess) {
		if let Some(current) = access.document_of_position(self.position) {
			if let Some(document) = access.document(current) {
				self.position = document.token_start;
			}
		}
	}

	pub fn to_document_end(&mut self, access: &DaemonAccess) {
		if let Some(current) = access.document_of_position(self.position) {
			if let Some(document) = access.document(current) {
				self.position = document.token_end.saturating_sub(1).max(document.token_start);
			}
		}
	}
}
