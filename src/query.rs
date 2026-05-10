use anyhow::Result;
use montre_index::Corpus;
use montre_query::executor::Results;

pub struct QueryBar {
	pub text: String,
	pub cursor: usize,
}

impl QueryBar {
	pub fn new() -> Self {
		Self {
			text: String::new(),
			cursor: 0,
		}
	}

	pub fn move_cursor_to_end(&mut self) {
		self.cursor = self.text.len();
	}

	pub fn move_cursor_left(&mut self) {
		if self.cursor == 0 {
			return;
		}
		let mut new_position = self.cursor - 1;
		while new_position > 0 && !self.text.is_char_boundary(new_position) {
			new_position -= 1;
		}
		self.cursor = new_position;
	}

	pub fn move_cursor_right(&mut self) {
		if self.cursor >= self.text.len() {
			return;
		}
		let mut new_position = self.cursor + 1;
		while new_position < self.text.len() && !self.text.is_char_boundary(new_position) {
			new_position += 1;
		}
		self.cursor = new_position;
	}

	pub fn insert_char(&mut self, character: char) {
		self.text.insert(self.cursor, character);
		self.cursor += character.len_utf8();
	}

	pub fn backspace(&mut self) {
		if self.cursor == 0 {
			return;
		}
		let mut previous = self.cursor - 1;
		while previous > 0 && !self.text.is_char_boundary(previous) {
			previous -= 1;
		}
		self.text.replace_range(previous..self.cursor, "");
		self.cursor = previous;
	}
}

pub fn run_query(corpus: &Corpus, cql: &str) -> Result<Results> {
	let parsed = montre_query::parse(cql)?;
	let plan = montre_query::planner::plan(&parsed)?;
	let mut results = montre_query::executor::execute(&plan, corpus)?;
	results.populate_context(corpus);
	Ok(results)
}
