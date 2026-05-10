use montre_index::{Corpus, SpanIndex};
use montre_query::executor::Results;

pub struct KwicLine {
	pub document: String,
	pub sentence_id: String,
	pub left: String,
	pub hit: String,
	pub right: String,
	pub hit_position: u64,
}

pub fn format_kwic(corpus: &Corpus, results: &Results, window_tokens: usize) -> Vec<KwicLine> {
	results
		.hits()
		.iter()
		.map(|hit| {
			let sentence = corpus.spans().containing("sentence", hit.span.start);
			let sentence_start = sentence.map(|span| span.start).unwrap_or(0);
			let sentence_end = sentence.map(|span| span.end).unwrap_or(u64::MAX);

			let window = window_tokens as u64;
			let left_start = hit.span.start.saturating_sub(window).max(sentence_start);
			let right_end = (hit.span.end + window).min(sentence_end);

			let document = corpus
				.document_names()
				.get(hit.document_index as usize)
				.cloned()
				.unwrap_or_default();
			let sentence_id = corpus
				.sentence_id(hit.sentence_index as usize)
				.map(|id| id.to_string())
				.unwrap_or_default();

			KwicLine {
				document,
				sentence_id,
				left: corpus.surface_text(left_start, hit.span.start),
				hit: corpus.surface_text(hit.span.start, hit.span.end),
				right: corpus.surface_text(hit.span.end, right_end),
				hit_position: hit.span.start,
			}
		})
		.collect()
}
