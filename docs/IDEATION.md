# AgentEyeAi — Ideation (formerly AgentMonitorAi)

Desktop app to monitor AI agents: their sessions, context, and source activity. Priority: **speed across large volumes of agent sources and sessions**.

## 1. Problem & goals

- See all agent sessions (Claude Code, Codex CLI, Gemini CLI, Cursor, Copilot, custom agents) in one place.
- Live view: what is each agent doing *right now*, how full is its context window, what is it costing.
- Historical view: search/filter/replay thousands of sessions instantly.
- Context inspection: what is in the context (system prompt, files read, tool results, compactions), token breakdown by category.
- Stay out of the way: read-only, low CPU/RAM, no impact on the agents being watched.

### Non-goals (for v1)
- Controlling or steering agents (read-only first).
- Cloud sync / multi-user.

## 2. Where the data lives (the "sources")

Most agents already write local logs; monitoring = tailing + indexing them.

| Agent | Typical location | Format |
|---|---|---|
| Claude Code | `~/.claude/projects/<proj>/<session>.jsonl` | JSONL, append-only |
| Codex CLI | `~/.codex/sessions/**/rollout-*.jsonl` | JSONL |
| Gemini CLI | `~/.gemini/tmp/**` | JSON/JSONL |
| Cursor / others | app-data SQLite / JSON | varies |
| Custom agents | OpenTelemetry (OTLP) / hooks / stdout | pluggable |

Key properties that drive the design:
- Files are **append-only JSONL** → incremental parsing by byte offset; never re-read whole files.
- Files can be **huge** (tens/hundreds of MB, single lines of several MB).
- Many files, constantly changing → need an efficient file watcher + debounce.
- Two ingestion modes: **passive** (watch log files) and **active** (agent hooks / OTLP endpoint pushing events). Start passive.

## 3. Architecture sketch

```
 Source adapters (one per agent type)
   └─ watcher (notify / ReadDirectoryChangesW) ─► tailer (offset-based) ─► parser
                                                                         │ normalized events
                                                                         ▼
                                              SQLite (WAL) + FTS5 index  ◄── single writer
                                                                         │
                                  query API (read-only connections) ◄────┘
                                                                         │ push deltas
                                                                         ▼
                                                                    UI (virtualized)
```

- **Normalized event model**: `session`, `message`, `tool_call`, `tool_result`, `usage` (input/output/cache tokens), `compaction`, `file_ref`. Adapters map each agent's format into this.
- **Store raw offsets, not raw content, where possible**: index metadata + FTS text in SQLite; keep a pointer (file, byte range) to the original line and lazily load full payloads on demand. Keeps the DB small and ingestion fast.
- **Context tracking**: derive context fill per turn from usage records (input + cache read/creation tokens vs. model window); mark compaction boundaries.
- **Performance rules**: parse in a background thread pool, batch inserts in transactions, backpressure on UI updates (coalesce to ~10–30 Hz), virtualized lists/timelines, lazy-load big tool results.

## 4. Stack options

### A. Tauri 2 (Rust core + web UI) — **recommended**

- Rust core: fast JSONL parsing (`serde_json`/`simd-json`), `notify` watcher, `rusqlite` + FTS5, `rayon`/tokio for ingestion.
- Small footprint (~10–30 MB installer, low idle RAM, uses WebView2 on Windows).
- UI in Svelte/React/Solid with a virtualized list (TanStack Virtual) and a charting lib (uPlot/ECharts) for token/context timelines.
- Cons: Rust learning curve; WebView2 differences on macOS/Linux if going cross-platform.

### B. .NET + Avalonia (C# end to end)

- Native-feeling, fast, one language, great tooling; `System.Text.Json` / `Utf8JsonReader` is fast enough; `Microsoft.Data.Sqlite` + FTS5.
- Good fit if you're already in the .NET ecosystem. Virtualized `ItemsRepeater`/`DataGrid` handles large lists well.
- Cons: charting/timeline ecosystem smaller than web; styling more work.

### C. Electron + TypeScript

- Fastest to prototype, richest UI ecosystem, easiest hiring/community.
- Cons: heavy RAM (~200–400 MB), JSON parsing of huge logs should go to a worker or native addon; "monitor that is itself a hog" is ironic.

### D. Wails (Go core + web UI)

- Go gives easy concurrency and a small binary; simpler than Rust. Smaller ecosystem than Tauri, same WebView2 UI approach.

### E. Rust-native UI (egui / Slint / Iced)

- Maximum speed and one language, but timelines/rich text/markdown rendering are much more work. Probably overkill.

### Comparison

| | Ingestion speed | UI richness | RAM | Dev speed | Notes |
| --- | --- | --- | --- | --- | --- |
| Tauri (Rust+web) | ★★★★★ | ★★★★★ | ★★★★ | ★★★ | Best speed/UX balance |
| Avalonia (.NET) | ★★★★ | ★★★ | ★★★★ | ★★★★ | Best if C# is home turf |
| Electron | ★★★ | ★★★★★ | ★★ | ★★★★★ | Fastest MVP |
| Wails (Go) | ★★★★ | ★★★★ | ★★★★ | ★★★★ | Simpler than Rust |
| egui/Slint | ★★★★★ | ★★ | ★★★★★ | ★★ | Heavy UI lift |

**Recommendation:** Tauri 2 + Rust core + SQLite/FTS5 + Svelte (or React) UI. The "fast across sources and sessions" requirement is mainly an *ingestion/indexing/search* problem, which Rust handles best, while the web UI makes timelines and rich transcripts easy. If you'd rather stay in C#, Avalonia is the strongest alternative.

## 5. Feature ideas

**MVP**
1. Auto-discover agent log dirs; list projects → sessions (sorted by recency, with live "active" indicator).
2. Live session view: streaming transcript, tool calls collapsed by default.
3. Context gauge per session: tokens used vs. window, breakdown (system / user / assistant / tool results / cache), compaction markers.
4. Global full-text search across all sessions (FTS5) with filters (agent, project, date, model, tool).
5. Cost/token totals per session/day/project.

**Next**
- Timeline/flame view of a session (turn latency, tool durations, parallel subagents).
- Subagent / sidechain tree.
- Diff of what files an agent read/edited; jump to file.
- Alerts: context > 80%, runaway loops, repeated failing tool calls, stuck/idle agents, cost thresholds.
- Session compare, bookmarks/tags, export (Markdown/JSON).
- Active ingestion: local OTLP/HTTP endpoint + hook scripts for agents that don't log.
- Tray icon with live status of running agents.
- Redaction of secrets in the UI/export.

## 6. Risks & open design questions

- **Format drift**: agent log formats change without notice → versioned adapters, tolerant parsing, fixture-based tests from real logs.
- **Huge lines / files**: stream-parse, cap in-memory payloads, lazy-load.
- **File-watch reliability on Windows** (network/synced folders such as Nextcloud): add a periodic rescan fallback.
- **Privacy**: logs contain code and secrets → local-only, no telemetry, optional redaction.
- **Accurate token counts**: rely on reported usage when present; estimate otherwise and label as estimated.
- **Locking**: open log files read-only with shared access so agents are never blocked.

## 7. Suggested next steps

1. Pick the stack (see questions below).
2. Spike: parse a real Claude Code session JSONL → SQLite + FTS5, measure ingest throughput on your actual `~/.claude` directory (target: > 100 MB/s parse, search < 50 ms).
3. Define the normalized event schema and the adapter trait/interface.
4. Build the live watcher + minimal session list/transcript UI.
5. Add context gauge, then search, then alerts.

## 8. Decisions

- **Agents (v1):** Claude Code and Codex CLI. Cursor and Gemini CLI dropped for now. Others (Copilot, custom) later.
- **Ingestion model: on-demand, tiered.**
  1. Discovery: stat plus first/last lines only (title, cwd, model, timestamps). The session list needs no index.
  2. Open session: parse that file in memory, nothing persisted. Active sessions keep reading appended bytes.
  3. Search: a background, low-priority, resumable FTS index (per-file byte offset). Files not yet indexed are scanned on the fly, so results are always complete. The index can be switched off or wiped.
- **Search:** in scope for v1 (derived FTS index, rebuildable from the source files).
- **Platforms:** Windows and Linux.
- **Stack:** Rust + Tauri 2. UI framework: Angular with signals (Tauri is frontend-agnostic; it only needs static build output).
- **Ingestion:** passive log/session watching plus an active channel (agent hooks / OTLP).
- **Storage:** store only paths/offsets to sessions; read context on the fly from the source files. SQLite holds the index (and FTS text if we keep search), not copies of content.

## 9. Still open

- Rust ingest throughput spike (the Python scan below is only a baseline).

## 10. Data scale spike (this machine, 2026-10-31)

| Agent | Session files | Size | Lines | Max line | Median file |
|---|---|---|---|---|---|
| Claude Code (`~/.claude/projects`) | 66 `.jsonl` (+6 subagent) | 274 MB | 66.6k | ~957 KB | ~1.4 MB (max 50 MB) |
| Codex (`~/.codex/sessions`) | 172 | 156 MB | 55.5k | ~815 KB | ~0.6 MB (max 9 MB) |
| Gemini | none | — | — | — | — |

- Total about 430 MB, 240 files, 122k lines. Every line parsed as valid JSON.
- Plain Python `json` parsed at about 105-117 MB/s, so a full cold index here takes a few seconds even before Rust. Rust with `serde_json` should be several times faster.
- Design the app for 10-100x this (heavy users reach tens of GB). Incremental offset-based tailing matters more than raw parse speed.
- Claude record types seen: assistant, user, attachment, last-prompt, ai-title, api-request(-blob), plus others. Codex: response_item, event_msg, token_usage_record, turn_context, compacted, session_meta.
- Search implication: the text is about 430 MB, so an FTS5 index is likely 100-300 MB, a fraction of the source. Fine to keep as a rebuildable cache. Index message text only, and skip the large tool results and blobs.
