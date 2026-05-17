use std::path::Path;
use std::sync::mpsc::Receiver;

use anyhow::Result;
use montre_tui_core::daemon::client::NotificationEnvelope;
use montre_tui_core::protocol::{
	CorpusDocumentsParams, CorpusInfo, CouplerCreateParams, CouplerKind, Hit, Interest,
	InterestKind, ProcessInfo, ProcessKind, PublishInterestParams, QueryDiscardParams,
	QueryExecuteParams, QueryExecuteReply, QueryHitsParams, RegisterParams,
	SessionRosterParams, SubscriptionParams, TextSurfaceParams,
};
use montre_tui_core::DaemonClient;

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
			kind: ProcessKind::Kwic,
			label: Some("montre-kwic".to_string()),
			provides: vec![InterestKind::Hit],
			consumes: vec![],
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

	pub fn query_execute(&mut self, cql: String) -> Result<QueryExecuteReply> {
		Ok(self.client.query_execute(QueryExecuteParams { cql })?)
	}

	pub fn query_hits(&mut self, handle: &str, offset: u64, limit: u64) -> Result<Vec<Hit>> {
		let reply = self.client.query_hits(QueryHitsParams {
			handle: handle.to_string(),
			offset,
			limit,
		})?;
		Ok(reply.hits)
	}

	pub fn query_discard(&mut self, handle: &str) -> Result<()> {
		self.client.query_discard(QueryDiscardParams {
			handle: handle.to_string(),
		})?;
		Ok(())
	}

	pub fn text_surface(&mut self, start: u64, end: u64) -> Result<String> {
		let reply = self.client.text_surface(TextSurfaceParams { start, end })?;
		Ok(reply.surface)
	}

	pub fn publish_interest(&mut self, interest: Interest) -> Result<()> {
		self.client
			.publish_interest(PublishInterestParams { interest })?;
		Ok(())
	}

	pub fn subscribe_roster(&mut self) -> Result<()> {
		self.client.subscription_subscribe(SubscriptionParams {
			topic: "roster_changed".to_string(),
		})?;
		Ok(())
	}

	pub fn session_roster(&mut self) -> Result<Vec<ProcessInfo>> {
		let reply = self.client.roster(SessionRosterParams { filter: None })?;
		Ok(reply.processes)
	}

	pub fn coupler_create(
		&mut self,
		master_id: u32,
		follower_id: u32,
		kind: CouplerKind,
	) -> Result<u32> {
		let reply = self.client.coupler_create(CouplerCreateParams {
			master_id,
			follower_id,
			kind,
		})?;
		Ok(reply.coupler_id)
	}
}
