use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

use crate::app::App;
use crate::slots::{FocusTarget, ReaderState, SlotContent};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
	let corpus_name = app
		.corpus_path
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("?");

	let reader = focused_reader(app);

	let component = reader
		.and_then(|state| {
			if app.corpus.is_multi_component() {
				app.corpus
					.component_for_document(state.cursor.document_index as usize)
					.map(|component| component.name.clone())
			} else {
				None
			}
		})
		.unwrap_or_else(|| "—".to_string());

	let document = reader
		.and_then(|state| {
			app.corpus
				.document_names()
				.get(state.cursor.document_index as usize)
				.cloned()
		})
		.unwrap_or_else(|| "—".to_string());

	let sentence_id = reader
		.and_then(|state| app.corpus.sentence_id(state.cursor.sentence_index as usize))
		.map(|id| id.to_string())
		.unwrap_or_else(|| "—".to_string());

	let text = format!(
		" {} │ {} │ {} │ {} │ — ",
		corpus_name, component, document, sentence_id
	);
	let line = Line::from(text).style(app.theme.status_bar);
	frame.render_widget(line, area);
}

fn focused_reader(app: &App) -> Option<&ReaderState> {
	let slot = match app.focus {
		FocusTarget::TopSlot(index) => app.pane_layout.top_slots.get(index),
		FocusTarget::BottomSlot => app.pane_layout.bottom_slot.as_ref(),
	}?;
	if let SlotContent::Reader(state) = slot {
		Some(state)
	} else {
		None
	}
}
