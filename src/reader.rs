use montre_core::Span;
use montre_index::{Corpus, SpanIndex};

pub struct SentenceView {
	pub sentence_index: u32,
	pub span: Span,
	pub surface: String,
}

pub fn document_sentence_range(corpus: &Corpus, document_index: u32) -> Option<(u32, u32)> {
	let document_spans = corpus.spans().spans("document")?;
	let document = document_spans.get(document_index as usize)?;
	let sentence_spans = corpus.spans().spans("sentence")?;
	let first = sentence_spans.partition_point(|sentence| sentence.start < document.start);
	let one_past_last = sentence_spans.partition_point(|sentence| sentence.start < document.end);
	Some((first as u32, one_past_last as u32))
}

pub fn sentences_in_range(corpus: &Corpus, start: u32, end: u32) -> Vec<SentenceView> {
	let Some(sentence_spans) = corpus.spans().spans("sentence") else {
		return Vec::new();
	};
	let start = start as usize;
	let end = (end as usize).min(sentence_spans.len());
	if start >= end {
		return Vec::new();
	}
	(start..end)
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
