# montre-reader

Status: Parked pending engine-side work. Read this before resuming.

## What this crate is

The token-stream, paragraph-faithful **reader**. Cursor is a global `TokenPosition` (u64); it fetches a token window via `text.surface_with_token_spans`, owns its line wrapping, and renders a coupler-driven span highlight. This is *not* the sentence-browser — see the last section.

## Current state

Slice 1 (token-stream navigation foundation) plus the case-1 newline fix. Builds and runs against the current daemon, but renders an unbroken token stream because the current corpus's surface is a synthetic space-join with no real whitespace. It is correct code blocked on a data/engine contract, not broken code.

Blocked on: the daemon serving real (whitespace-faithful) surface with tokens aligned to byte offsets in that surface. Full requirements and the proposed op are in `docs/engine-surface-and-position-requirements.md`. The moment surface carries real `\n`s, this reader renders paragraphs and line breaks with no further change here.

## Restoring this state in git

Slice 1 is commit `d57f58e`, which was then reverted. To bring it back, undo the revert — `git revert <revert-commit>` (the revert is current HEAD), or `git reset --hard d57f58e` if you'd rather drop the revert commit and haven't pushed.

One catch: the **case-1 newline fix landed after `d57f58e` and was never committed**, so undoing the revert restores slice 1 *without* it. Re-apply the provided `render.rs` (the canonical slice-1 + case-1 version) on top. The fix is confined to `render.rs`; every other file equals `d57f58e`.

## Load-bearing facts (the things that will surprise you otherwise)

- **Coupler**: registers `consumes: [Span]`; `CouplerUpdate` arm matches `Interest::Span { start, end, .. }`, sets cursor to `start`, stores `[start, end)` as the persistent highlight.
- **Unverified protocol assumption** (most likely break point): `daemon_access::from_client` calls `text.document` per doc and reads `detail.span.{start,end}` as the document's **token** span to build the position→document index. This reply shape was inferred from the protocol doc, not seen in compiled code. If the build fails in `from_client`, this is why — confirm `TextDocumentReply`'s field is `span: Span` in global token positions (Open question 3 in the engine doc).
- **kwic gate**: `montre-kwic` `auto_couple` requires `consumes(Span)`. Keep it on `Span` (slice 1 changed it from `Sentence`); reverting it silently stops kwic from coupling to this reader, which is the whole point of the follower.
- **Dependency**: `unicode-width` (workspace + this crate) is used by the wrap pass for display-cell-accurate columns. Don't drop it.
- **Deleted module**: slice 1 removed `montre-tui-core/src/sentence_source.rs` (orphaned once the reader stopped being sentence-addressed). The sentence-browser revives it from history.
- **Stale-target footgun**: a core module was deleted, so if a build behaves as if using stale code, `rm -rf target`.

## case-1 newline behavior and forward-compat

`build_layout` splits surface on `\n` into hard lines, wraps each independently, and emits an empty row per blank line (so `\n\n` reads as a paragraph gap). `position_by_row_delta` steps over empty rows so Down/Up don't stall at a gap. This is shape-agnostic: it reflects whatever whitespace the surface contains, so it works the same for hand-injected `# newpar`, UDPipe `NewPar`, and TEI, and renders verse/hard line breaks for free. Deliberately *not* done: collapsing runs of blank lines, and trimming a leading/trailing blank row — both are easy adds if the engine's real surface turns out to need them. Tabs are not treated as breaks (only `\n` and space are special); revisit if real surface carries `\t`.

## Relationship to sentence-browser

Separate utility, not built here. It is the pre-slice-1, sentence-addressed reader (`Cursor { document_index, sentence_within_document }`), kept as an unadvertised dev/demo follower for testing kwic etc. Its open task is its own coupler bridge: it must consume `Span` and translate an incoming global position back to a sentence (it currently has no token coordinates). That bridge depends on Open questions 1–4 in the engine doc. When built, it lives in its own crate and revives `sentence_source` from history. Do not conflate the two: this crate is the paragraph-faithful reader; the sentence-browser is the fidelity-limited fallback for corpora without `NewPar`.
