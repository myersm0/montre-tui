# montre-tui

Terminal UI for [Montre](https://github.com/myersm0/montre), a corpus query engine for parallel and aligned corpora.

## Status

Very early development, not yet usable. Check back soon.

## Design overview

Configurable slot-based layout: 1–3 top slots hosting readers, document pickers, vocabulary browsers, or stats; an optional bottom slot defaulting to KWIC results; a persistent query strip; an info bar; and a context-sensitive key hints bar.

Per-slot cursors with anchoring relations support parallel reading across components without special-casing — single-language reading, two-language parallel, and three-language aligned reading all fall out of the same model.

Built on ratatui with a synchronous event loop. Links Montre directly as a Rust dependency rather than via the C FFI.

## Build

```bash
cargo run --release -- /path/to/corpus
```

Requires a built Montre corpus at the given path.

## Keys

`q` quits. More to come.

## License

Apache-2.0
