use montre_index::Corpus;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::kwic;
use crate::slots::KwicState;
use crate::theme::Theme;

pub fn draw(
	frame: &mut Frame,
	area: Rect,
	state: &KwicState,
	is_focused: bool,
	corpus: &Corpus,
	theme: &Theme,
) {
	let border = if is_focused {
		&theme.pane_border_active
	} else {
		&theme.pane_border_inactive
	};

	let title = match (&state.results, &state.error) {
		(Some(results), _) => format!(" KWIC ({} hits) ", results.hits().len()),
		(None, Some(_)) => " KWIC (error) ".to_string(),
		(None, None) => " KWIC ".to_string(),
	};

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(border.border_type)
		.border_style(border.style)
		.title(Span::styled(title, theme.pane_title));

	let inner = block.inner(area);
	frame.render_widget(block, area);

	if let Some(error) = &state.error {
		let line = Line::from(Span::styled(error.clone(), theme.error));
		let paragraph = Paragraph::new(Text::from(line)).wrap(Wrap { trim: false });
		frame.render_widget(paragraph, inner);
		return;
	}

	let Some(results) = &state.results else {
		let placeholder = Line::from(Span::styled(
			"Press : to enter a query",
			theme.hints_bar,
		));
		frame.render_widget(placeholder, inner);
		return;
	};

	if results.hits().is_empty() {
		let placeholder = Line::from(Span::styled("No matches.", theme.hints_bar));
		frame.render_widget(placeholder, inner);
		return;
	}

	let lines = kwic::format_kwic(corpus, results, state.window_tokens);
	let inner_height = inner.height as usize;
	let top_margin = 2;
	let scroll = state.selected.saturating_sub(top_margin);

	let visible: Vec<Line> = lines
		.iter()
		.enumerate()
		.skip(scroll)
		.take(inner_height)
		.map(|(index, line)| {
			let is_selected = index == state.selected;
			let row_style = if is_selected {
				theme.kwic_selected
			} else {
				theme.text_default
			};
			Line::from(vec![
				Span::styled(format!("{:<24}", truncate(&line.document, 24)), theme.kwic_meta),
				Span::styled(format!("{:<14}", truncate(&line.sentence_id, 14)), theme.kwic_meta),
				Span::styled(format!("{:>30}", truncate_left(&line.left, 30)), theme.kwic_left),
				Span::styled(format!(" {} ", line.hit), theme.kwic_match),
				Span::styled(line.right.clone(), theme.kwic_right),
			])
			.style(row_style)
		})
		.collect();

	let paragraph = Paragraph::new(Text::from(visible));
	frame.render_widget(paragraph, inner);
}

fn truncate(text: &str, width: usize) -> String {
	if text.chars().count() <= width {
		text.to_string()
	} else {
		let mut result: String = text.chars().take(width.saturating_sub(1)).collect();
		result.push('…');
		result
	}
}

fn truncate_left(text: &str, width: usize) -> String {
	let count = text.chars().count();
	if count <= width {
		text.to_string()
	} else {
		let skip = count - width.saturating_sub(1);
		let mut result = String::from("…");
		result.extend(text.chars().skip(skip));
		result
	}
}
