mod cursor;
mod daemon_access;
mod render;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use montre_tui_core::daemon::client::NotificationEnvelope;
use montre_tui_core::palette::Palette;
use montre_tui_core::protocol::{Interest, Span};
use montre_tui_core::runtime;
use montre_tui_core::terminal::{self, ManagedTerminal};
use montre_tui_core::theme::Theme;

use crate::cursor::Cursor;
use crate::daemon_access::{DaemonAccess, TokenWindow};
use crate::render::{Mode, ViewState};

#[allow(non_upper_case_globals)]
const window_back: u64 = 1024;
#[allow(non_upper_case_globals)]
const window_forward: u64 = 1024;
#[allow(non_upper_case_globals)]
const window_margin: u64 = 512;

#[derive(Parser)]
#[command(name = "montre-reader", about = "Token-stream reader for Montre corpora")]
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
	let theme = Theme::from_palette(&Palette::grundtvig_dark());

	let mut terminal = terminal::init()?;
	let result = run_loop(&mut terminal, access, theme);
	terminal::restore(&mut terminal)?;
	result
}

struct App {
	access: DaemonAccess,
	cursor: Cursor,
	window: TokenWindow,
	highlight: Option<Span>,
	theme: Theme,
	mode: Mode,
	connected: bool,
	shutdown_initiated_at: Option<Instant>,
	dirty: bool,
	quit: bool,
}

fn run_loop(terminal: &mut ManagedTerminal, mut access: DaemonAccess, theme: Theme) -> Result<()> {
	let mut cursor = Cursor::at_corpus_start();
	let window = build_window(&mut access, cursor.position)?;
	if let Some(position) = window.emitted_at_or_after(cursor.position) {
		cursor.position = position;
	}

	let mut app = App {
		access,
		cursor,
		window,
		highlight: None,
		theme,
		mode: Mode::Normal,
		connected: true,
		shutdown_initiated_at: None,
		dirty: true,
		quit: false,
	};

	loop {
		if app.quit {
			break;
		}
		if let Some(started_at) = app.shutdown_initiated_at {
			if started_at.elapsed() >= runtime::shutdown_grace {
				break;
			}
		}

		if event::poll(runtime::poll_interval)? {
			match event::read()? {
				Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(&mut app, key),
				Event::Resize(_, _) => app.dirty = true,
				_ => {}
			}
		}

		let (pending, disconnected) = runtime::drain_notifications(app.access.notifications());
		for notification in pending {
			handle_notification(&mut app, notification);
		}
		if disconnected && app.shutdown_initiated_at.is_none() {
			begin_shutdown(&mut app, "connection lost".to_string());
		}

		if app.dirty {
			let view = ViewState {
				cursor_position: app.cursor.position,
				window: &app.window,
				highlight: app.highlight.as_ref(),
				mode: &app.mode,
				connected: app.connected,
				theme: &app.theme,
			};
			let mut render_result: Result<()> = Ok(());
			terminal.draw(|frame| {
				render_result = render::draw(frame, &app.access, &view);
			})?;
			render_result?;
			app.dirty = false;
		}
	}
	Ok(())
}

fn build_window(access: &mut DaemonAccess, center: u64) -> Result<TokenWindow> {
	let start = center.saturating_sub(window_back);
	let end = center + window_forward;
	access.fetch_window(start, end)
}

fn ensure_window(app: &mut App) {
	let cursor = app.cursor.position;
	let near_low = app.window.first_position().map(|first| cursor < first + window_margin).unwrap_or(true);
	let near_high = app.window.last_position().map(|last| cursor + window_margin > last).unwrap_or(true);
	let at_corpus_start = app.window.first_position().map(|first| first == 0).unwrap_or(false);
	let at_corpus_end = app
		.window
		.last_position()
		.map(|last| last + 1 >= app.access.token_ceiling())
		.unwrap_or(false);
	if (near_low && !at_corpus_start) || (near_high && !at_corpus_end) {
		if let Ok(window) = build_window(&mut app.access, cursor) {
			app.window = window;
		}
	}
}

fn move_token(app: &mut App, forward: bool) {
	let next = if forward {
		app.window.next_emitted(app.cursor.position)
	} else {
		app.window.prev_emitted(app.cursor.position)
	};
	if let Some(position) = next {
		app.cursor.position = position;
		ensure_window(app);
		app.dirty = true;
	}
}

fn move_rows(app: &mut App, delta: isize) {
	let layout = render::build_layout(&app.window, inner_width());
	if let Some(position) = layout.position_by_row_delta(app.cursor.position, delta) {
		app.cursor.position = position;
		ensure_window(app);
		app.dirty = true;
	}
}

fn jumped(app: &mut App) {
	ensure_window(app);
	let position = app.cursor.position;
	if let Some(snapped) = app
		.window
		.emitted_at_or_before(position)
		.or_else(|| app.window.emitted_at_or_after(position))
	{
		app.cursor.position = snapped;
	}
	app.dirty = true;
}

fn inner_width() -> usize {
	crossterm::terminal::size().map(|(cols, _)| cols.saturating_sub(2) as usize).unwrap_or(78)
}

fn inner_height() -> usize {
	crossterm::terminal::size().map(|(_, rows)| rows.saturating_sub(4) as usize).unwrap_or(20)
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
		KeyCode::Left | KeyCode::Char('h') => move_token(app, false),
		KeyCode::Right | KeyCode::Char('l') => move_token(app, true),
		KeyCode::Down | KeyCode::Char('j') => move_rows(app, 1),
		KeyCode::Up | KeyCode::Char('k') => move_rows(app, -1),
		KeyCode::PageDown => move_rows(app, inner_height() as isize),
		KeyCode::PageUp => move_rows(app, -(inner_height() as isize)),
		KeyCode::Char('J') => {
			app.cursor.advance_document(&app.access);
			jumped(app);
		}
		KeyCode::Char('K') => {
			app.cursor.retreat_document(&app.access);
			jumped(app);
		}
		KeyCode::Char(']') => {
			app.cursor.advance_component(&app.access);
			jumped(app);
		}
		KeyCode::Char('[') => {
			app.cursor.retreat_component(&app.access);
			jumped(app);
		}
		KeyCode::Home | KeyCode::Char('g') => {
			app.cursor.to_document_start(&app.access);
			jumped(app);
		}
		KeyCode::End | KeyCode::Char('G') => {
			app.cursor.to_document_end(&app.access);
			jumped(app);
		}
		_ => {}
	}
}

fn handle_notification(app: &mut App, notification: NotificationEnvelope) {
	match notification {
		NotificationEnvelope::CouplerUpdate { interest, .. } => {
			if let Interest::Span { start, end, .. } = interest {
				app.cursor.set_position(start, &app.access);
				app.highlight = Some(Span { start, end });
				ensure_window(app);
				app.dirty = true;
			}
		}
		NotificationEnvelope::Shutdown { reason, .. } => {
			begin_shutdown(app, reason.to_string());
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
