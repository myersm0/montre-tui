use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod app;
mod cursor;
mod keyhints;
mod kwic;
mod query;
mod reader;
mod slots;
mod theme;
mod ui;

#[derive(Parser)]
#[command(name = "montre-tui", version)]
struct Cli {
	corpus_path: PathBuf,
}

fn main() -> Result<()> {
	let cli = Cli::parse();
	let corpus = montre_index::open(&cli.corpus_path)?;
	let mut application = app::App::new(corpus, cli.corpus_path);
	application.run()
}
