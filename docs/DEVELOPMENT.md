
## Architecture
 
```
                      ┌──────────────────────────┐
                      │ Daemon (montre serve)    │
                      │                          │
                      │ Owns: corpus, couplers,  │
                      │ named results, query     │
                      │ history, process roster, │
                      │ workspace state          │
                      └─────────┬────────────────┘
                                │ Unix socket (JSON-RPC)
        ┌──────────────┬────────┴──────┬──────────────┐
        │              │               │              │
   ┌────┴─────┐  ┌─────┴─────┐  ┌──────┴────┐  ┌──────┴─────┐
   │ montre   │  │ montre    │  │ montre    │  │ Julia REPL │
   │ reader   │  │ kwic      │  │ conllu    │  │ via Montre │
   │          │  │           │  │           │  │ .connect() │
   └──────────┘  └───────────┘  └───────────┘  └────────────┘
```
 
The daemon is the integration point. It loads the corpus once and shares it across all connected clients via a local Unix socket. Every client — TUI binary, Julia, eventually Python — speaks the same JSON-RPC protocol. Clients announce themselves on connect, receive a process ID, optionally subscribe to topics, and can declare coupling relationships with other connected clients.
 
One daemon per corpus, scoped to that corpus's lifetime. Auto-spawned on first client connect, idle-shutdown after the last client disconnects. No deployment, no external services, no network — Unix domain sockets only.
 
## Couplers
 
The daemon tracks **couplers**: typed relationships between two connected clients in which one's selection drives the other's view. When a `montre kwic` user moves the cursor onto a hit, that selection flows through a coupler to a connected `montre reader`, which scrolls to and highlights the hit's exact token span. The reader doesn't know about kwic; it knows it has a *master* publishing a span of interest, and it follows. The span is the hit's literal `[start, end)`, so a match that straddles sentence boundaries projects faithfully rather than being widened to a containing sentence.
 
The term is borrowed from the pipe organ. In organ building, the *montre* is the rank of pipes standing on the visible face of the instrument; *couplers* are the mechanisms that link ranks together so that playing one rank sounds others in concert. The TUI works similarly: each binary is its own rank, and couplers are how the organist puts them in concert.
 
Couplers are typed — a `KwicSelection` coupler projects the hit's exact span, an `Alignment` coupler projects a span across parallel components, a `SentenceMirror` follows a sentence. Setting them up is interactive (a key in each binary opens coupler management); the daemon enforces compatibility between what masters publish and what followers consume.


## Theming
 
The TUI theme is built in two layers: a **palette** of color tokens and a **theme** that maps UI roles to those tokens.
 
`Palette` (in `montre-tui-core/src/palette.rs`) holds raw colors as tokens: `page`, `elevated`, `recessed`, `border_subtle`, `border_strong`, `text_muted`, `text_body`, `text_strong`, `brick`, `honey`, `verdigris`, plus seven `highlight_*` tokens. Each variant constructor (e.g. `Palette::grundtvig_dark()`) supplies values. Adding a new palette is one new constructor.
 
`Theme` (in `theme.rs`) holds one `Style` per UI role: borders, text, status bar, hints, overlays, etc. `Theme::from_palette(&Palette)` constructs the theme by mapping each role to the appropriate token. To retheme, change the palette; the role mappings stay put.
 
The palette has a separate set of `highlight_*` tokens that hold values from the *inverse* mode. In `grundtvig_dark`, the `highlight_*` tokens come from the *light* variant of `grundtvig-mono`; in a future `grundtvig_light`, they'd come from the dark variant. Selected rows in the UI use these inverse-mode tokens, so a highlight reads as a literal bright patch against the dark page — the contrast model is "selection borrows from the opposite palette."
 
The `RowStyles` helper bundles per-row styles for a row that may or may not be highlighted:
 
```rust
let styles = theme.row_styles(is_selected);
Line::from(vec![
    Span::styled(text, styles.text_default),
    Span::styled(more_text, styles.text_subtle),
])
.style(styles.background)
```
 
`row_styles` returns `{ background, text_default, text_subtle, kwic_match, cursor_marker }`. Binaries that render rows (kwic hit rows, reader sentence lines, planned `docs`/`vocab`/`results` browsers) destructure the result instead of branching on `is_selected` per span.
 
### Page background
 
The page background is painted by every binary as the first thing in its `draw()`:
 
```rust
frame.render_widget(Block::default().style(theme.page_background), frame.area());
```
 
Without this fill, the palette's `page` token only applies to cells that explicitly set a bg (selected row, input cursor, overlay). Every other cell falls through to terminal default. The fill makes the buffer start as the palette's `page` color; subsequent widgets render on top.
 
## Core modules
 
Library code lives in `montre-tui-core`. The crate consists of free functions and helpers. Each binary keeps its own event loop, App struct, key dispatch, and daemon-access wrapper.
 
**What's in core:**
 
| Module           | Purpose                                                  |
| ---------------- | -------------------------------------------------------- |
| `palette`        | Color tokens and palette constructors.                   |
| `theme`          | Role-to-token mapping plus `RowStyles` helper.           |
| `terminal`       | `init()`, `restore(&mut)`, `ManagedTerminal` type alias. |
| `overlay`        | `centered_rect`, `draw_help`, `draw_shutdown`.           |
| `runtime`        | `poll_interval`, `shutdown_grace`, `drain_notifications`. |
| `status_bar`     | `draw_status_bar` and `StatusBarContent`.                |
| `key_hint`       | `draw_hints_bar` and `KeyHint`.                          |
| `sentence_source`| `SentenceSource` trait and `SentenceView` struct.        |
| `daemon`         | Re-export of `montre-daemon`.                            |
 
**What stays per-binary:**
 
- `Mode` enums — kwic has `Edit`; future binaries will add their own variants. Lifting would require a trait, which crosses from helper into framework.
- `App` struct and `handle_key` dispatch — reference binary-specific state.
- `DaemonAccess` — same shape but per-binary caches diverge enough that abstraction is more pain than gain at two binaries. Worth revisiting at three.

## Conventions
 
A new binary's `main.rs` follows this shape:
 
```rust
fn main() -> Result<()> {
    // ... parse args, connect to daemon ...
    let theme = Theme::from_palette(&Palette::grundtvig_dark());
    let mut terminal = terminal::init()?;
    let result = run_loop(&mut terminal, access, theme);
    terminal::restore(&mut terminal)?;
    result
}
 
fn run_loop(terminal: &mut ManagedTerminal, /* ... */) -> Result<()> {
    let mut app = App::new(/* ... */);
    loop {
        if app.quit { break; }
        if let Some(started_at) = app.shutdown_initiated_at {
            if started_at.elapsed() >= runtime::shutdown_grace { break; }
        }
 
        if event::poll(runtime::poll_interval)? {
            // ... handle event ...
        }
 
        let (pending, disconnected) = runtime::drain_notifications(
            app.access.notifications()
        );
        for notification in pending {
            handle_notification(&mut app, notification);
        }
        if disconnected && app.shutdown_initiated_at.is_none() {
            begin_shutdown(&mut app, "connection lost".to_string());
        }
 
        if app.dirty {
            terminal.draw(|frame| {
                let _ = render::draw(frame, /* ... */);
            })?;
            app.dirty = false;
        }
    }
    Ok(())
}
```
 
A new binary's `render.rs` follows this shape:
 
```rust
pub fn draw(frame: &mut Frame, /* ... */) -> Result<()> {
    frame.render_widget(Block::default().style(theme.page_background), frame.area());
    // ... lay out regions ...
    match view.mode {
        Mode::Normal => {}
        Mode::Help => overlay::draw_help(frame, frame.area(), theme, "<binary> help", &help_entries()),
        Mode::ShuttingDown { reason } => overlay::draw_shutdown(frame, frame.area(), reason, theme),
        // ... binary-specific modes ...
    }
    Ok(())
}
 
fn help_entries() -> &'static [(&'static str, &'static str)] {
    &[
        // ... key, description pairs ...
    ]
}
```
 
