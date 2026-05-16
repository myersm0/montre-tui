use std::path::Path;
use std::sync::mpsc::Receiver;

use anyhow::Result;
use montre_tui_core::protocol::{
	CorpusDocumentsParams, CorpusInfo, Interest, InterestKind, ProcessKind, PublishInterestParams,
	RegisterParams, TextSentenceParams, TextSentencesParams,
};
use montre_tui_core::sentence_source::{DocumentMeta, SentenceSource, SentenceView};
use montre_tui_core::DaemonClient;
use montre_tui_core::daemon::client::NotificationEnvelope;

pub struct DocumentSummary {
	pub index: u32,
	pub name: String,
	pub component: String,
	pub sentence_count: u32,
}

pub struct DaemonAccess {
	client: DaemonClient,
	pub process_id: u32,
	pub daemon_epoch: u64,
	pub corpus_info: CorpusInfo,
	pub documents: Vec<DocumentSummary>,
}

impl DaemonAccess {
	pub fn connect_or_spawn(corpus_path: &Path) -> Result<Self> {
		let client = DaemonClient::connect_or_spawn(corpus_path)?;
		Self::from_client(client)
	}

	pub fn connect_socket(socket_path: &Path) -> Result<Self> {
		let client = DaemonClient::connect(socket_path)?;
		Self::from_client(client)
	}

	fn from_client(mut client: DaemonClient) -> Result<Self> {
		let session = client.register(RegisterParams {
			protocol_version: 1,
			kind: ProcessKind::Reader,
			label: Some("montre-reader".to_string()),
			provides: vec![],
			consumes: vec![InterestKind::Sentence],
		})?;

		let corpus_info = client.corpus_info()?;
		let document_list = client.corpus_documents(CorpusDocumentsParams { component: None })?;

		let documents: Vec<DocumentSummary> = document_list
			.documents
			.into_iter()
			.map(|document| DocumentSummary {
				index: document.index,
				name: document.name,
				component: document.component,
				sentence_count: document.sentence_count,
			})
			.collect();

		for (position, summary) in documents.iter().enumerate() {
			debug_assert_eq!(summary.index as usize, position);
		}

		Ok(Self {
			client,
			process_id: session.process_id,
			daemon_epoch: session.daemon_epoch,
			corpus_info,
			documents,
		})
	}

	pub fn notifications(&self) -> &Receiver<NotificationEnvelope> {
		self.client.notifications()
	}

	pub fn document_count(&self) -> u32 {
		self.documents.len() as u32
	}

	pub fn document(&self, document_index: u32) -> Option<&DocumentSummary> {
		self.documents.get(document_index as usize)
	}

	pub fn document_meta(&self, document_index: u32) -> Option<DocumentMeta> {
		let summary = self.document(document_index)?;
		Some(DocumentMeta {
			index: summary.index,
			name: summary.name.clone(),
			component: summary.component.clone(),
			sentence_count: summary.sentence_count,
		})
	}

	pub fn is_multi_component(&self) -> bool {
		self.corpus_info.components.len() > 1
	}

	pub fn component_of_document(&self, document_index: u32) -> Option<&str> {
		self.document(document_index)
			.map(|summary| summary.component.as_str())
	}

	pub fn first_document_of_next_component(&self, document_index: u32) -> Option<u32> {
		let current_component = self.component_of_document(document_index)?.to_string();
		self.documents
			.iter()
			.find(|summary| summary.component != current_component && summary.index > document_index)
			.map(|summary| summary.index)
	}

	pub fn first_document_of_previous_component(&self, document_index: u32) -> Option<u32> {
		let current_component = self.component_of_document(document_index)?.to_string();
		let previous_component = self
			.documents
			.iter()
			.rev()
			.find(|summary| summary.component != current_component && summary.index < document_index)
			.map(|summary| summary.component.clone())?;
		self.documents
			.iter()
			.find(|summary| summary.component == previous_component)
			.map(|summary| summary.index)
	}

	pub fn to_global_sentence(&self, document_index: u32, sentence_within_document: u32) -> u64 {
		let mut total: u64 = 0;
		for summary in &self.documents[..document_index as usize] {
			total += summary.sentence_count as u64;
		}
		total + sentence_within_document as u64
	}

	pub fn from_global_sentence(&self, global: u64) -> (u32, u32) {
		let mut remaining = global;
		for summary in &self.documents {
			let count = summary.sentence_count as u64;
			if remaining < count {
				return (summary.index, remaining as u32);
			}
			remaining -= count;
		}
		match self.documents.last() {
			Some(last) => (last.index, last.sentence_count.saturating_sub(1)),
			None => (0, 0),
		}
	}

	pub fn total_sentences(&self) -> u64 {
		self.documents
			.iter()
			.map(|summary| summary.sentence_count as u64)
			.sum()
	}

	#[allow(dead_code)]
	pub fn publish_interest(&mut self, interest: Interest) -> Result<()> {
		self.client
			.publish_interest(PublishInterestParams { interest })?;
		Ok(())
	}
}

impl SentenceSource for DaemonAccess {
	fn sentence(&mut self, document_index: u32, sentence_within_document: u32) -> Result<SentenceView> {
		let result = self.client.text_sentence(TextSentenceParams {
			doc: document_index,
			sent: sentence_within_document,
		})?;
		Ok(SentenceView {
			sentence_within_document,
			span: result.span,
			surface: result.surface,
			sentence_id: result.sentence_id,
		})
	}

	fn sentences(
		&mut self,
		document_index: u32,
		sentence_start: u32,
		sentence_end: u32,
	) -> Result<Vec<SentenceView>> {
		if sentence_start >= sentence_end {
			return Ok(Vec::new());
		}
		let result = self.client.text_sentences(TextSentencesParams {
			doc: document_index,
			sent_start: sentence_start,
			sent_end: sentence_end,
		})?;
		Ok(result
			.sentences
			.into_iter()
			.map(|sentence| SentenceView {
				sentence_within_document: sentence.sent,
				span: sentence.span,
				surface: sentence.surface,
				sentence_id: sentence.sentence_id,
			})
			.collect())
	}
}
