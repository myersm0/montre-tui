use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use montre_index::Corpus;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::path::PathBuf;
use std::sync::Arc;

use crate::query::{self, QueryBar};
use crate::slots::{FocusTarget, PaneLayout, ReaderState, SlotContent};
use crate::theme::Theme;
use crate::ui;

const PAGE_STEP: usize = 10;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Mode {
	Normal,
	QueryEntry,
}

pub struct App {
	pub corpus: Arc<Corpus>,
	pub corpus_path: PathBuf,
	pub pane_layout: PaneLayout,
	pub focus: FocusTarget,
	pub theme: Theme,
	pub mode: Mode,
	pub query_bar: QueryBar,
	pub should_quit: bool,
}

impl App {
	pub fn new(corpus: Corpus, corpus_path: PathBuf) -> Self {
		Self {
			corpus: Arc::new(corpus),
			corpus_path,
			pane_layout: PaneLayout::new(),
			focus: FocusTarget::TopSlot(0),
			theme: Theme::default_dark(),
			mode: Mode::Normal,
			query_bar: QueryBar::new(),
			should_quit: false,
		}
	}

	pub fn run(&mut self) -> Result<()> {
		let mut terminal = setup_terminal()?;
		let result = self.event_loop(&mut terminal);
		restore_terminal(&mut terminal)?;
		result
	}

	fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
		while !self.should_quit {
			terminal.draw(|frame| ui::draw(frame, self))?;
			if let Event::Key(key) = event::read()? {
				if key.kind == KeyEventKind::Press {
					self.handle_key(key.code);
				}
			}
		}
		Ok(())
	}

	fn handle_key(&mut self, key: KeyCode) {
		match self.mode {
			Mode::Normal => self.handle_key_normal(key),
			Mode::QueryEntry => self.handle_key_query_entry(key),
		}
	}

	fn handle_key_normal(&mut self, key: KeyCode) {
		if matches!(key, KeyCode::Char('q')) {
			self.should_quit = true;
			return;
		}

		match key {
			KeyCode::Char(':') => {
				self.mode = Mode::QueryEntry;
				self.query_bar.move_cursor_to_end();
				return;
			}
			KeyCode::Tab => {
				self.focus = self.pane_layout.cycle_focus_forward(self.focus);
				return;
			}
			KeyCode::BackTab => {
				self.focus = self.pane_layout.cycle_focus_backward(self.focus);
				return;
			}
			KeyCode::Char('+') => {
				self.focus = self.pane_layout.open_slot_after(self.focus);
				return;
			}
			KeyCode::Char('-') => {
				self.focus = self.pane_layout.close_slot(self.focus);
				return;
			}
			KeyCode::Char('r') => {
				let focus = self.focus;
				let is_empty = matches!(
					self.pane_layout.focused_slot(focus),
					Some(SlotContent::Empty)
				);
				if is_empty {
					self.pane_layout
						.set_focused_content(focus, SlotContent::Reader(ReaderState::new()));
				}
				return;
			}
			_ => {}
		}

		let focus = self.focus;
		if let Some(state) = self.pane_layout.focused_reader_mut(focus) {
			let corpus: &Corpus = &self.corpus;
			match key {
				KeyCode::Up => state.cursor.retreat_sentence(corpus),
				KeyCode::Down => state.cursor.advance_sentence(corpus),
				KeyCode::PageUp => state.cursor.retreat_screen(corpus, PAGE_STEP),
				KeyCode::PageDown => state.cursor.advance_screen(corpus, PAGE_STEP),
				KeyCode::Home => state.cursor.to_document_start(corpus),
				KeyCode::End => state.cursor.to_document_end(corpus),
				KeyCode::Char('J') => state.cursor.advance_document(corpus),
				KeyCode::Char('K') => state.cursor.retreat_document(corpus),
				KeyCode::Char(']') => state.cursor.advance_component(corpus),
				KeyCode::Char('[') => state.cursor.retreat_component(corpus),
				_ => {}
			}
			return;
		}

		if let Some(state) = self.pane_layout.focused_kwic_mut(focus) {
			match key {
				KeyCode::Up => state.select_previous(),
				KeyCode::Down => state.select_next(),
				_ => {}
			}
		}
	}

	fn handle_key_query_entry(&mut self, key: KeyCode) {
		match key {
			KeyCode::Esc => {
				self.mode = Mode::Normal;
			}
			KeyCode::Enter => {
				self.execute_current_query();
				self.mode = Mode::Normal;
			}
			KeyCode::Char(character) => self.query_bar.insert_char(character),
			KeyCode::Backspace => self.query_bar.backspace(),
			KeyCode::Left => self.query_bar.move_cursor_left(),
			KeyCode::Right => self.query_bar.move_cursor_right(),
			KeyCode::Home => self.query_bar.cursor = 0,
			KeyCode::End => self.query_bar.move_cursor_to_end(),
			_ => {}
		}
	}

	fn execute_current_query(&mut self) {
		let cql = self.query_bar.text.clone();
		let outcome = query::run_query(&self.corpus, &cql);
		let kwic = self.pane_layout.ensure_kwic_slot();
		match outcome {
			Ok(results) => {
				kwic.results = Some(results);
				kwic.error = None;
				kwic.selected = 0;
				kwic.scroll_offset = 0;
			}
			Err(error) => {
				kwic.results = None;
				kwic.error = Some(error.to_string());
				kwic.selected = 0;
				kwic.scroll_offset = 0;
			}
		}
	}
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
	enable_raw_mode()?;
	let mut output = stdout();
	execute!(output, EnterAlternateScreen)?;
	Ok(Terminal::new(CrosstermBackend::new(output))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
	disable_raw_mode()?;
	execute!(stdout(), LeaveAlternateScreen)?;
	terminal.show_cursor()?;
	Ok(())
}
