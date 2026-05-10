use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::app::App;
use crate::slots::{FocusTarget, SlotContent};
use crate::theme::Theme;
use montre_index::Corpus;

mod hints_bar;
mod info_bar;
mod query_bar;
mod reader_pane;

pub fn draw(frame: &mut Frame, app: &App) {
	let regions = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Min(0),
			Constraint::Length(1),
			Constraint::Length(1),
			Constraint::Length(1),
		])
		.split(frame.area());

	draw_top_slots(frame, regions[0], app);
	query_bar::draw(frame, regions[1], app);
	info_bar::draw(frame, regions[2], app);
	hints_bar::draw(frame, regions[3], app);
}

fn draw_top_slots(frame: &mut Frame, area: Rect, app: &App) {
	let slot_count = app.pane_layout.top_slots.len();
	let constraints: Vec<Constraint> = (0..slot_count)
		.map(|_| Constraint::Ratio(1, slot_count as u32))
		.collect();
	let slot_areas = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(constraints)
		.split(area);

	for (slot_index, slot) in app.pane_layout.top_slots.iter().enumerate() {
		let is_focused = matches!(app.focus, FocusTarget::TopSlot(active) if active == slot_index);
		draw_slot(
			frame,
			slot_areas[slot_index],
			slot,
			is_focused,
			&app.corpus,
			&app.theme,
		);
	}
}

fn draw_slot(
	frame: &mut Frame,
	area: Rect,
	slot: &SlotContent,
	is_focused: bool,
	corpus: &Corpus,
	theme: &Theme,
) {
	match slot {
		SlotContent::Empty => {
			let border = if is_focused {
				&theme.pane_border_active
			} else {
				&theme.pane_border_inactive
			};
			let block = Block::default()
				.borders(Borders::ALL)
				.border_type(border.border_type)
				.border_style(border.style);
			frame.render_widget(block, area);
		}
		SlotContent::Reader(state) => {
			reader_pane::draw(frame, area, state, is_focused, corpus, theme);
		}
	}
}
