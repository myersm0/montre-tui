# montre-kwic

The interactive query and concordance binary for [`montre-tui`](../..).

A KWIC ("keyword in context") view renders each hit of a query as a row: the matched span between aligned columns of left and right context, plus document and sentence identifiers. `montre-kwic` pairs that view with a query bar so the same window does both — write a CQL query, scan the results, move the cursor onto an interesting hit. Coupled followers (typically `montre-reader`) update as the cursor moves.

## Running

```bash
montre kwic <corpus-path>
```

Opens the corpus, auto-spawning the daemon if it isn't already running.

## Layout

Four horizontal regions, top to bottom: an always-visible query bar, the concordance body, a one-line key-hint row, and a status bar. The concordance body uses fixed columns — document, sentence, left context (right-aligned), match (bold), right context (left-aligned) — the classical KWIC convention.

## Keys

**Normal mode** (concordance navigation):

- `j` / `k` / `↑` / `↓` — previous / next hit
- `PgUp` / `PgDn` — page through hits
- `g` / `G`, `Home` / `End` — first / last hit
- `i` — focus the query bar
- `Enter` — republish current hit (useful for late-joining followers)
- `?` — help
- `q` — quit

**Edit mode** (query bar):

- typing — insert text
- `←` / `→`, `Home` / `End` — move within the line
- `Backspace` — delete char before cursor
- `Enter` — execute query, return to Normal mode
- `Esc` — cancel, return to Normal mode

## Coupling

`montre-kwic` publishes `Hit` interest: the cursor row is the current hit, and any cursor move notifies coupled followers, which resolve it to a sentence span. For now, the binary auto-creates `KwicSelection` couplers to every connected process that consumes `Sentence` (typically `montre-reader`). Interactive coupler management is planned.

## Limitations

- First 100 hits per query. A banner reports when more exist; paging keys are pending.
- Single result handle at a time — a new query discards the previous result. Named-result storage and recall (`:save` / `:load`) is planned.
- Fixed column widths. Wide matches or rich context may clip on narrow terminals.
