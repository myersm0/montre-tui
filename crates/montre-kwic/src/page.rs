pub struct HitsPage {
	pub handle: String,
	pub hit_count: u64,
	pub rows: Vec<HitRow>,
	pub cursor: usize,
}

pub struct HitRow {
	pub document_index: u32,
	pub sentence_index: u32,
	pub match_text: String,
}
