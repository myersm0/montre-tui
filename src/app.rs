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

use crate::slots::{FocusTarget, PaneLayout};
use crate::theme::Theme;
use crate::ui;

pub struct App {
	pub corpus: Arc<Corpus>,
	pub corpus_path: PathBuf,
	pub pane_layout: PaneLayout,
	pub focus: FocusTarget,
	pub theme: Theme,
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
		match key {
			KeyCode::Char('q') => self.should_quit = true,
			_ => {}
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
