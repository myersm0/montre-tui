
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
 
The daemon tracks **couplers**: typed relationships between two connected clients in which one's selection drives the other's view. When a `montre kwic` user moves the cursor onto a hit, that selection flows through a coupler to a connected `montre reader`, which scrolls to the matching sentence. The reader doesn't know about kwic; it knows it has a *master* publishing a sentence of interest, and it follows.
 
The term is borrowed from the pipe organ. In organ building, the *montre* is the rank of pipes standing on the visible face of the instrument; *couplers* are the mechanisms that link ranks together so that playing one rank sounds others in concert. The TUI works similarly: each binary is its own rank, and couplers are how the organist puts them in concert.
 
Couplers are typed — a `KwicSelection` coupler carries a hit, an `Alignment` coupler projects a span across parallel components, a `SentenceMirror` follows a sentence. Setting them up is interactive (a key in each binary opens coupler management); the daemon enforces compatibility between what masters publish and what followers consume.
