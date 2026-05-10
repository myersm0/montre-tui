# montre-tui

Terminal interfaces for [Montre](https://github.com/myersm0/montre), a corpus query engine for aligned and parallel corpora.

`montre-tui` is built around a local per-corpus session daemon (`montre serve`) coordinating multiple independent terminal clients: readers, KWIC browsers, CoNLL-U inspectors, document pickers, vocabulary, amed-results browsers, and Julia/Python REPL sessions.

Rather than embedding a full multipane UI in a single process, each view runs in its own terminal pane, split, or window. Your favorite terminal multiplexer (e.g. tmux) or window manager handles layout; the daemon handles shared state, anchors, named results, and query history.

The result is a lightweight, composable workflow that you could set up as follows, for example:
- one pane following a French text
- another showing aligned English sentences
- another browsing KWIC results
- another inspecting raw CoNLL-U annotations
- a Julia or Python REPL connected to the same corpus session for statistics and visualizations

## Status

Very early development, not yet usable. Check back soon.

## Planned binaries

- `montre reader` — document reader
- `montre kwic` — interactive concordance browser and query interface
- `montre conllu` — CoNLL-U inspector
- `montre docs` — document picker
- `montre vocab` — vocabulary browser
- `montre results` — named-results browser
- `montre tui` — optional convenience launcher for tmux/zellij layouts

## Design principles

- Corpus data immutable and self-contained
- Shared session state through a local daemon
- Typed anchoring relationships between tools
- No external services or deployment
- Unix-native composition with terminals, REPLs, editors, etc
- Reader-first workflows
- Voice accessibility features throughout

## Architecture

Clients communicate with the daemon over a local Unix socket using a JSON-RPC protocol.

## Build

```bash
cargo build --release
```

The daemon lives in the main montre repository.

## License

Apache-2.0

