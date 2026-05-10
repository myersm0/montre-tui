use montre_index::{Corpus, SpanIndex};

pub struct Cursor {
	pub document_index: u32,
	pub sentence_index: u32,
}

impl Cursor {
	pub fn advance_sentence(&mut self, corpus: &Corpus) {
		let total = total_sentences(corpus);
		if (self.sentence_index as usize) + 1 < total {
			self.sentence_index += 1;
			if let Some(document) = document_for_sentence(corpus, self.sentence_index) {
				self.document_index = document;
			}
		}
	}

	pub fn retreat_sentence(&mut self, corpus: &Corpus) {
		if self.sentence_index > 0 {
			self.sentence_index -= 1;
			if let Some(document) = document_for_sentence(corpus, self.sentence_index) {
				self.document_index = document;
			}
		}
	}

	pub fn advance_screen(&mut self, corpus: &Corpus, height: usize) {
		let total = total_sentences(corpus);
		if total == 0 {
			return;
		}
		let target = ((self.sentence_index as usize) + height).min(total - 1);
		self.sentence_index = target as u32;
		if let Some(document) = document_for_sentence(corpus, self.sentence_index) {
			self.document_index = document;
		}
	}

	pub fn retreat_screen(&mut self, corpus: &Corpus, height: usize) {
		let target = (self.sentence_index as usize).saturating_sub(height);
		self.sentence_index = target as u32;
		if let Some(document) = document_for_sentence(corpus, self.sentence_index) {
			self.document_index = document;
		}
	}

	pub fn advance_document(&mut self, corpus: &Corpus) {
		let document_count = corpus.document_names().len() as u32;
		if self.document_index + 1 < document_count {
			self.document_index += 1;
			if let Some(first) = first_sentence_of_document(corpus, self.document_index) {
				self.sentence_index = first;
			}
		}
	}

	pub fn retreat_document(&mut self, corpus: &Corpus) {
		if self.document_index > 0 {
			self.document_index -= 1;
			if let Some(first) = first_sentence_of_document(corpus, self.document_index) {
				self.sentence_index = first;
			}
		}
	}

	pub fn advance_component(&mut self, corpus: &Corpus) {
		let components = corpus.components();
		if components.is_empty() {
			return;
		}
		if let Some(current) = corpus.component_for_document(self.document_index as usize) {
			let next_id = current.id + 1;
			if let Some(next) = components.get(next_id as usize) {
				self.document_index = next.document_range.0 as u32;
				if let Some(first) = first_sentence_of_document(corpus, self.document_index) {
					self.sentence_index = first;
				}
			}
		}
	}

	pub fn retreat_component(&mut self, corpus: &Corpus) {
		let components = corpus.components();
		if components.is_empty() {
			return;
		}
		if let Some(current) = corpus.component_for_document(self.document_index as usize) {
			if current.id > 0 {
				let previous_id = current.id - 1;
				if let Some(previous) = components.get(previous_id as usize) {
					self.document_index = previous.document_range.0 as u32;
					if let Some(first) = first_sentence_of_document(corpus, self.document_index) {
						self.sentence_index = first;
					}
				}
			}
		}
	}

	pub fn to_document_start(&mut self, corpus: &Corpus) {
		if let Some(first) = first_sentence_of_document(corpus, self.document_index) {
			self.sentence_index = first;
		}
	}

	pub fn to_document_end(&mut self, corpus: &Corpus) {
		if let Some(last) = last_sentence_of_document(corpus, self.document_index) {
			self.sentence_index = last;
		}
	}
}

fn total_sentences(corpus: &Corpus) -> usize {
	corpus
		.spans()
		.spans("sentence")
		.map(|spans| spans.len())
		.unwrap_or(0)
}

fn first_sentence_of_document(corpus: &Corpus, document_index: u32) -> Option<u32> {
	let document_spans = corpus.spans().spans("document")?;
	let sentence_spans = corpus.spans().spans("sentence")?;
	let document = document_spans.get(document_index as usize)?;
	let first = sentence_spans.partition_point(|sentence| sentence.start < document.start);
	Some(first as u32)
}

fn last_sentence_of_document(corpus: &Corpus, document_index: u32) -> Option<u32> {
	let document_spans = corpus.spans().spans("document")?;
	let sentence_spans = corpus.spans().spans("sentence")?;
	let document = document_spans.get(document_index as usize)?;
	let one_past_last = sentence_spans.partition_point(|sentence| sentence.start < document.end);
	if one_past_last == 0 {
		None
	} else {
		Some((one_past_last - 1) as u32)
	}
}

fn document_for_sentence(corpus: &Corpus, sentence_index: u32) -> Option<u32> {
	let sentence_spans = corpus.spans().spans("sentence")?;
	let sentence = sentence_spans.get(sentence_index as usize)?;
	let document_spans = corpus.spans().spans("document")?;
	let after = document_spans.partition_point(|document| document.start <= sentence.start);
	if after == 0 {
		None
	} else {
		Some((after - 1) as u32)
	}
}
