use std::path::Path;
use std::sync::mpsc::Receiver;

use anyhow::Result;
use montre_tui_core::daemon::client::NotificationEnvelope;
use montre_tui_core::protocol::{
	CorpusDocumentsParams, CorpusInfo, Interest, InterestKind, ProcessKind,
	PublishInterestParams, RegisterParams, Span, SurfaceToken, TextDocumentParams,
	TextSurfaceWithTokenSpansParams,
};
use montre_tui_core::DaemonClient;

pub struct DocumentSummary {
	pub index: u32,
	pub name: String,
	pub component: String,
	pub sentence_count: u32,
	pub token_start: u64,
	pub token_end: u64,
}

pub struct TokenWindow {
	pub start: u64,
	pub surface: String,
	pub tokens: Vec<SurfaceToken>,
}

impl TokenWindow {
	pub fn first_position(&self) -> Option<u64> {
		self.tokens.first().map(|token| token.position)
	}

	pub fn last_position(&self) -> Option<u64> {
		self.tokens.last().map(|token| token.position)
	}

	pub fn next_emitted(&self, position: u64) -> Option<u64> {
		self.tokens
			.iter()
			.find(|token| token.emitted && token.position > position)
			.map(|token| token.position)
	}

	pub fn prev_emitted(&self, position: u64) -> Option<u64> {
		self.tokens
			.iter()
			.rev()
			.find(|token| token.emitted && token.position < position)
			.map(|token| token.position)
	}

	pub fn emitted_at_or_after(&self, position: u64) -> Option<u64> {
		self.tokens
			.iter()
			.find(|token| token.emitted && token.position >= position)
			.map(|token| token.position)
	}

	pub fn emitted_at_or_before(&self, position: u64) -> Option<u64> {
		self.tokens
			.iter()
			.rev()
			.find(|token| token.emitted && token.position <= position)
			.map(|token| token.position)
	}
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
			consumes: vec![InterestKind::Span],
		})?;

		let corpus_info = client.corpus_info()?;
		let document_list = client.corpus_documents(CorpusDocumentsParams { component: None })?;

		let mut documents = Vec::with_capacity(document_list.documents.len());
		for document in document_list.documents {
			let detail = client.text_document(TextDocumentParams { doc: document.index })?;
			documents.push(DocumentSummary {
				index: document.index,
				name: document.name,
				component: document.component,
				sentence_count: document.sentence_count,
				token_start: detail.span.start,
				token_end: detail.span.end,
			});
		}

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

	pub fn document(&self, document_index: u32) -> Option<&DocumentSummary> {
		self.documents.get(document_index as usize)
	}

	pub fn is_multi_component(&self) -> bool {
		self.corpus_info.components.len() > 1
	}

	pub fn component_of_document(&self, document_index: u32) -> Option<&str> {
		self.document(document_index).map(|summary| summary.component.as_str())
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

	pub fn token_ceiling(&self) -> u64 {
		self.documents.last().map(|summary| summary.token_end).unwrap_or(0)
	}

	pub fn document_of_position(&self, position: u64) -> Option<u32> {
		let found = self.documents.binary_search_by(|summary| {
			if position < summary.token_start {
				std::cmp::Ordering::Greater
			} else if position >= summary.token_end {
				std::cmp::Ordering::Less
			} else {
				std::cmp::Ordering::Equal
			}
		});
		found.ok().map(|index| self.documents[index].index)
	}

	pub fn fetch_window(&mut self, start: u64, end: u64) -> Result<TokenWindow> {
		let reply = self.client.text_surface_with_token_spans(TextSurfaceWithTokenSpansParams {
			ranges: vec![Span { start, end }],
		})?;
		let (surface, tokens) = match reply.results.into_iter().next() {
			Some(result) => (result.surface, result.tokens),
			None => (String::new(), Vec::new()),
		};
		Ok(TokenWindow { start, surface, tokens })
	}

	#[allow(dead_code)]
	pub fn publish_interest(&mut self, interest: Interest) -> Result<()> {
		self.client.publish_interest(PublishInterestParams { interest })?;
		Ok(())
	}
}
