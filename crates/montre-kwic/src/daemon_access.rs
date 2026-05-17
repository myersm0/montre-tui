use std::path::Path;
use std::sync::mpsc::Receiver;

use anyhow::Result;
use montre_tui_core::protocol::{CorpusInfo, InterestKind, ProcessKind, RegisterParams};
use montre_tui_core::DaemonClient;
use montre_tui_core::daemon::client::NotificationEnvelope;

pub struct DaemonAccess {
	client: DaemonClient,
	pub process_id: u32,
	pub daemon_epoch: u64,
	pub corpus_info: CorpusInfo,
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
		Ok(Self {
			client,
			process_id: session.process_id,
			daemon_epoch: session.daemon_epoch,
			corpus_info,
		})
	}

	pub fn notifications(&self) -> &Receiver<NotificationEnvelope> {
		self.client.notifications()
	}
}
