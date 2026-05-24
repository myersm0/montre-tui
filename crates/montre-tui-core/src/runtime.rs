use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use crate::daemon::client::NotificationEnvelope;

#[allow(non_upper_case_globals)]
pub const poll_interval: Duration = Duration::from_millis(50);

#[allow(non_upper_case_globals)]
pub const shutdown_grace: Duration = Duration::from_millis(500);

pub fn drain_notifications(
	receiver: &Receiver<NotificationEnvelope>,
) -> (Vec<NotificationEnvelope>, bool) {
	let mut pending = Vec::new();
	loop {
		match receiver.try_recv() {
			Ok(notification) => pending.push(notification),
			Err(TryRecvError::Empty) => return (pending, false),
			Err(TryRecvError::Disconnected) => return (pending, true),
		}
	}
}
