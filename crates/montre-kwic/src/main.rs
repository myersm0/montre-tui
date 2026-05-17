mod daemon_access;
mod page;
mod query_bar;
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
use montre_tui_core::theme::Theme;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::daemon_access::DaemonAccess;
use crate::page::{HitRow, HitsPage};
use crate::query_bar::QueryBar;
use crate::render::{Mode, ViewState};

#[derive(Parser)]
#[command(name = "montre-kwic", about = "Concordance browser and query interface for Montre corpora")]
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
	theme: Theme,
	mode: Mode,
	query_bar: QueryBar,
	page: Option<HitsPage>,
	error: Option<String>,
	connected: bool,
	shutdown_initiated_at: Option<Instant>,
	dirty: bool,
	quit: bool,
}

impl App {
	fn new(access: DaemonAccess, theme: Theme) -> Self {
		Self {
			access,
			theme,
			mode: Mode::Normal,
			query_bar: QueryBar::new(),
			page: None,
			error: None,
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
	let poll_interval = Duration::from_millis(50);
	let shutdown_grace = Duration::from_millis(500);
	let mut app = App::new(access, theme);

	loop {
		if app.quit {
			break;
		}
		if let Some(started_at) = app.shutdown_initiated_at {
			if started_at.elapsed() >= shutdown_grace {
				break;
			}
		}

		if event::poll(poll_interval)? {
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
				mode: &app.mode,
				query_bar: &app.query_bar,
				page: app.page.as_ref(),
				error: app.error.as_deref(),
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
		Mode::Edit => handle_edit_key(app, key),
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
		KeyCode::Char('i') => {
			app.mode = Mode::Edit;
			app.dirty = true;
		}
		KeyCode::Down | KeyCode::Char('j') => move_cursor(app, 1),
		KeyCode::Up | KeyCode::Char('k') => move_cursor(app, -1),
		KeyCode::PageDown => {
			let step = viewport_step_estimate() as isize;
			move_cursor(app, step);
		}
		KeyCode::PageUp => {
			let step = viewport_step_estimate() as isize;
			move_cursor(app, -step);
		}
		KeyCode::Home | KeyCode::Char('g') => jump_cursor(app, 0),
		KeyCode::End | KeyCode::Char('G') => {
			let last = app.page.as_ref().map(|page| page.rows.len().saturating_sub(1));
			if let Some(target) = last {
				jump_cursor(app, target);
			}
		}
		_ => {}
	}
}

fn handle_edit_key(app: &mut App, key: KeyEvent) {
	if key.modifiers.contains(KeyModifiers::CONTROL) {
		if matches!(key.code, KeyCode::Char('c')) {
			app.quit = true;
		}
		return;
	}
	match key.code {
		KeyCode::Esc => {
			app.mode = Mode::Normal;
			app.dirty = true;
		}
		KeyCode::Enter => {
			if !app.query_bar.text.trim().is_empty() {
				run_query(app);
				app.mode = Mode::Normal;
			}
			app.dirty = true;
		}
		KeyCode::Char(character) => {
			app.query_bar.insert_char(character);
			app.dirty = true;
		}
		KeyCode::Backspace => {
			app.query_bar.backspace();
			app.dirty = true;
		}
		KeyCode::Left => {
			app.query_bar.move_left();
			app.dirty = true;
		}
		KeyCode::Right => {
			app.query_bar.move_right();
			app.dirty = true;
		}
		KeyCode::Home => {
			app.query_bar.move_to_start();
			app.dirty = true;
		}
		KeyCode::End => {
			app.query_bar.move_to_end();
			app.dirty = true;
		}
		_ => {}
	}
}

fn move_cursor(app: &mut App, delta: isize) {
	let Some(page) = &mut app.page else {
		return;
	};
	if page.rows.is_empty() {
		return;
	}
	let last = page.rows.len() - 1;
	let target = (page.cursor as isize + delta).clamp(0, last as isize) as usize;
	if target != page.cursor {
		page.cursor = target;
		app.dirty = true;
	}
}

fn jump_cursor(app: &mut App, target: usize) {
	let Some(page) = &mut app.page else {
		return;
	};
	if page.rows.is_empty() {
		return;
	}
	let clamped = target.min(page.rows.len() - 1);
	if clamped != page.cursor {
		page.cursor = clamped;
		app.dirty = true;
	}
}

fn viewport_step_estimate() -> usize {
	crossterm::terminal::size()
		.map(|(_, rows)| rows.saturating_sub(4) as usize)
		.unwrap_or(20)
}

fn run_query(app: &mut App) {
	let cql = app.query_bar.text.trim().to_string();
	if cql.is_empty() {
		return;
	}
	let prior_handle = app.page.as_ref().map(|page| page.handle.clone());
	if let Some(handle) = prior_handle {
		let _ = app.access.query_discard(&handle);
	}
	app.page = None;
	app.error = None;
	match execute_query(&mut app.access, &cql) {
		Ok(page) => {
			app.page = Some(page);
		}
		Err(error) => {
			app.error = Some(error.to_string());
		}
	}
}

fn execute_query(access: &mut DaemonAccess, cql: &str) -> Result<HitsPage> {
	let reply = access.query_execute(cql.to_string())?;
	let limit: u64 = 100;
	let window_tokens: u64 = 10;
	let hits = access.query_hits(&reply.handle, 0, limit)?;
	let mut rows: Vec<HitRow> = Vec::with_capacity(hits.len());
	for hit in hits {
		let match_text = access.text_surface(hit.span.start, hit.span.end)?;
		let left_start = hit.span.start.saturating_sub(window_tokens);
		let left_text = access
			.text_surface(left_start, hit.span.start)
			.unwrap_or_default();
		let right_text = access
			.text_surface(hit.span.end, hit.span.end + window_tokens)
			.unwrap_or_default();
		rows.push(HitRow {
			document_index: hit.document_index,
			sentence_index: hit.sentence_index,
			left_text,
			match_text,
			right_text,
		});
	}
	Ok(HitsPage {
		handle: reply.handle,
		hit_count: reply.hit_count,
		rows,
		cursor: 0,
	})
}

fn handle_notification(app: &mut App, notification: NotificationEnvelope) {
	match notification {
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
