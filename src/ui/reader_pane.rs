use montre_index::Corpus;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::reader;
use crate::slots::ReaderState;
use crate::theme::Theme;

pub fn draw(
	frame: &mut Frame,
	area: Rect,
	state: &ReaderState,
	is_focused: bool,
	corpus: &Corpus,
	theme: &Theme,
) {
	let document_name = corpus
		.document_names()
		.get(state.cursor.document_index as usize)
		.cloned()
		.unwrap_or_default();

	let border = if is_focused {
		&theme.pane_border_active
	} else {
		&theme.pane_border_inactive
	};
	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(border.border_type)
		.border_style(border.style)
		.title(Span::styled(format!(" {} ", document_name), theme.pane_title));

	let inner = block.inner(area);
	frame.render_widget(block, area);

	let sentences = reader::sentences_in_document(corpus, state.cursor.document_index);
	let first_in_document = sentences
		.first()
		.map(|sentence| sentence.sentence_index)
		.unwrap_or(0);
	let cursor_offset = state
		.cursor
		.sentence_index
		.saturating_sub(first_in_document) as usize;

	let inner_height = inner.height as usize;
	let top_margin = (inner_height / 4).max(2);
	let scroll = cursor_offset.saturating_sub(top_margin);

	let lines: Vec<Line> = sentences
		.iter()
		.skip(scroll)
		.map(|sentence| {
			let is_cursor = sentence.sentence_index == state.cursor.sentence_index;
			let (marker, body_style) = if is_cursor {
				(
					Span::styled("▌ ", theme.cursor_marker),
					theme.cursor_sentence,
				)
			} else {
				(Span::raw("  "), theme.text_default)
			};
			Line::from(vec![marker, Span::styled(sentence.surface.clone(), body_style)])
		})
		.collect();

	let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
	frame.render_widget(paragraph, inner);
}
