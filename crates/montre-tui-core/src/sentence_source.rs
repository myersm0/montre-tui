use anyhow::Result;
use montre_daemon::protocol::Span;

pub struct SentenceView {
	pub sentence_within_document: u32,
	pub span: Span,
	pub surface: String,
	pub sentence_id: String,
}

pub struct DocumentMeta {
	pub index: u32,
	pub name: String,
	pub component: String,
	pub sentence_count: u32,
}

pub trait SentenceSource {
	fn sentence(&mut self, document_index: u32, sentence_within_document: u32) -> Result<SentenceView>;

	fn sentences(
		&mut self,
		document_index: u32,
		sentence_start: u32,
		sentence_end: u32,
	) -> Result<Vec<SentenceView>>;
}
