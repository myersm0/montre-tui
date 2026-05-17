mod cursor;
mod daemon_access;
mod render;

use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use montre_tui_core::daemon::client::NotificationEnvelope;
use montre_tui_core::protocol::Interest;
use montre_tui_core::theme::Theme;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::cursor::Cursor;
use crate::daemon_access::DaemonAccess;
use crate::render::{Mode, ViewState};

const POLL_INTERVAL: Duration = Duration::from_millis(50);
const SHUTDOWN_GRACE: Duration = Duration::from_millis(500);

#[derive(Parser)]
#[command(name = "montre-reader", about = "Single-document reader for Montre corpora")]
struct Cli {
	#[arg(required_unless_present = "socket")]
	corpus_path: Option<PathBuf>,

	#[arg(long, value_name = "PATH", help = "Connect to a daemon at this socket path instead of auto-spawning")]
	socket: Option<PathBuf>,
}

fn main() -> Result<()> {
	let cli = Cli::parse();
	let access = match cli.socket {
		Some(socket_path) => DaemonAccess::connect_socket(&socket_path)?,
		None => {
			let corpus_path = cli.corpus_path.expect("clap enforces presence when --socket is absent");
			let canonical = std::fs::canonicalize(&corpus_path)?;
			DaemonAccess::connect_or_spawn(&canonical)?
		}
	};
	let theme = Theme::default_dark();

	let mut terminal = init_terminal()?;
	let result = run_loop(&mut terminal, access, theme);
	restore_terminal(&mut terminal)?;
	result
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	terminal.show_cursor()?;
	Ok(())
}

struct App {
	access: DaemonAccess,
	cursor: Cursor,
	theme: Theme,
	mode: Mode,
	connected: bool,
	shutdown_initiated_at: Option<Instant>,
	dirty: bool,
	quit: bool,
}

impl App {
	fn new(access: DaemonAccess, theme: Theme) -> Self {
		Self {
			access,
			cursor: Cursor::at_corpus_start(),
			theme,
			mode: Mode::Normal,
			connected: true,
			shutdown_initiated_at: None,
			dirty: true,
			quit: false,
		}
	}
}

fn run_loop(
	terminal: &mut Terminal<CrosstermBackend<Stdout>>,
	access: DaemonAccess,
	theme: Theme,
) -> Result<()> {
	let mut app = App::new(access, theme);

	loop {
		if app.quit {
			break;
		}
		if let Some(started_at) = app.shutdown_initiated_at {
			if started_at.elapsed() >= SHUTDOWN_GRACE {
				break;
			}
		}

		if event::poll(POLL_INTERVAL)? {
			match event::read()? {
				Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(&mut app, key),
				Event::Resize(_, _) => app.dirty = true,
				_ => {}
			}
		}

		let mut pending: Vec<NotificationEnvelope> = Vec::new();
		let mut disconnected = false;
		{
			let notifications = app.access.notifications();
			loop {
				match notifications.try_recv() {
					Ok(notification) => pending.push(notification),
					Err(TryRecvError::Empty) => break,
					Err(TryRecvError::Disconnected) => {
						disconnected = true;
						break;
					}
				}
			}
		}
		for notification in pending {
			handle_notification(&mut app, notification);
		}
		if disconnected && app.shutdown_initiated_at.is_none() {
			begin_shutdown(&mut app, "connection lost".to_string());
		}

		if app.dirty {
			let view = ViewState {
				cursor: &app.cursor,
				mode: &app.mode,
				connected: app.connected,
				theme: &app.theme,
			};
			let mut render_result: Result<()> = Ok(());
			terminal.draw(|frame| {
				render_result = render::draw(frame, &mut app.access, &view);
			})?;
			render_result?;
			app.dirty = false;
		}
	}
	Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
	match &app.mode {
		Mode::ShuttingDown { .. } => {}
		Mode::Help => match key.code {
			KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
				app.mode = Mode::Normal;
				app.dirty = true;
			}
			_ => {}
		},
		Mode::Normal => handle_normal_key(app, key),
	}
}

fn handle_normal_key(app: &mut App, key: KeyEvent) {
	if key.modifiers.contains(KeyModifiers::CONTROL) {
		if matches!(key.code, KeyCode::Char('c')) {
			app.quit = true;
		}
		return;
	}
	match key.code {
		KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
		KeyCode::Char('?') => {
			app.mode = Mode::Help;
			app.dirty = true;
		}
		KeyCode::Down | KeyCode::Char('j') => {
			app.cursor.advance_sentence(&app.access);
			app.dirty = true;
		}
		KeyCode::Up | KeyCode::Char('k') => {
			app.cursor.retreat_sentence(&app.access);
			app.dirty = true;
		}
		KeyCode::PageDown => {
			let height = viewport_height_estimate();
			app.cursor.advance_screen(&app.access, height);
			app.dirty = true;
		}
		KeyCode::PageUp => {
			let height = viewport_height_estimate();
			app.cursor.retreat_screen(&app.access, height);
			app.dirty = true;
		}
		KeyCode::Char('J') => {
			app.cursor.advance_document(&app.access);
			app.dirty = true;
		}
		KeyCode::Char('K') => {
			app.cursor.retreat_document(&app.access);
			app.dirty = true;
		}
		KeyCode::Char(']') => {
			app.cursor.advance_component(&app.access);
			app.dirty = true;
		}
		KeyCode::Char('[') => {
			app.cursor.retreat_component(&app.access);
			app.dirty = true;
		}
		KeyCode::Home | KeyCode::Char('g') => {
			app.cursor.to_document_start();
			app.dirty = true;
		}
		KeyCode::End | KeyCode::Char('G') => {
			app.cursor.to_document_end(&app.access);
			app.dirty = true;
		}
		_ => {}
	}
}

fn viewport_height_estimate() -> usize {
	crossterm::terminal::size()
		.map(|(_, rows)| rows.saturating_sub(4) as usize)
		.unwrap_or(20)
}

fn handle_notification(app: &mut App, notification: NotificationEnvelope) {
	match notification {
		NotificationEnvelope::CouplerUpdate { interest, .. } => {
			if let Interest::Sentence { doc, sent } = interest {
				app.cursor.jump_to(doc, sent);
				app.dirty = true;
			}
		}
		NotificationEnvelope::Shutdown { reason, .. } => {
			begin_shutdown(app, reason);
		}
		_ => {}
	}
}

fn begin_shutdown(app: &mut App, reason: String) {
	app.mode = Mode::ShuttingDown { reason };
	app.connected = false;
	app.shutdown_initiated_at = Some(Instant::now());
	app.dirty = true;
}
