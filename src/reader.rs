use montre_core::Span;
use montre_index::{Corpus, SpanIndex};

pub struct SentenceView {
	pub sentence_index: u32,
	pub span: Span,
	pub surface: String,
}

pub fn sentences_in_document(corpus: &Corpus, document_index: u32) -> Vec<SentenceView> {
	let Some(document_spans) = corpus.spans().spans("document") else {
		return Vec::new();
	};
	let Some(document) = document_spans.get(document_index as usize) else {
		return Vec::new();
	};
	let Some(sentence_spans) = corpus.spans().spans("sentence") else {
		return Vec::new();
	};
	let first = sentence_spans.partition_point(|sentence| sentence.start < document.start);
	let last = sentence_spans.partition_point(|sentence| sentence.start < document.end);
	(first..last)
		.map(|index| {
			let span = sentence_spans[index];
			SentenceView {
				sentence_index: index as u32,
				span,
				surface: corpus.surface_text(span.start, span.end),
			}
		})
		.collect()
}
