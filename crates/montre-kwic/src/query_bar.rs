pub struct QueryBar {
	pub text: String,
	pub cursor_byte: usize,
}

impl QueryBar {
	pub fn new() -> Self {
		Self {
			text: String::new(),
			cursor_byte: 0,
		}
	}

	pub fn insert_char(&mut self, character: char) {
		self.text.insert(self.cursor_byte, character);
		self.cursor_byte += character.len_utf8();
	}

	pub fn backspace(&mut self) {
		if self.cursor_byte == 0 {
			return;
		}
		let mut previous = self.cursor_byte - 1;
		while previous > 0 && !self.text.is_char_boundary(previous) {
			previous -= 1;
		}
		self.text.replace_range(previous..self.cursor_byte, "");
		self.cursor_byte = previous;
	}

	pub fn move_left(&mut self) {
		if self.cursor_byte == 0 {
			return;
		}
		let mut new_position = self.cursor_byte - 1;
		while new_position > 0 && !self.text.is_char_boundary(new_position) {
			new_position -= 1;
		}
		self.cursor_byte = new_position;
	}

	pub fn move_right(&mut self) {
		if self.cursor_byte >= self.text.len() {
			return;
		}
		let mut new_position = self.cursor_byte + 1;
		while new_position < self.text.len() && !self.text.is_char_boundary(new_position) {
			new_position += 1;
		}
		self.cursor_byte = new_position;
	}

	pub fn move_to_start(&mut self) {
		self.cursor_byte = 0;
	}

	pub fn move_to_end(&mut self) {
		self.cursor_byte = self.text.len();
	}
}
