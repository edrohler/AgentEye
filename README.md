# AgentEye

Desktop app for browsing, searching and monitoring AI agent sessions (Claude Code and Codex CLI for now). Windows and Linux. Read-only: it never modifies agent files.

- Design notes and decisions: [docs/IDEATION.md](docs/IDEATION.md)
- Contributor and AI-agent guidelines (Rust, Angular, SOLID/DRY): [AGENTS.md](AGENTS.md)

## Stack

| Layer | Technology | Version |
| --- | --- | --- |
| Core | Rust (stable, edition 2021) | 1.99+ (workspace minimum 1.82) |
| Desktop shell | Tauri | 2.x |
| Frontend | Angular (standalone, signals, zoneless) | 22.x |
| Language | TypeScript | 6.0 |
| Runtime | Node.js | 24.15+ |
| Search index | SQLite FTS5 via `rusqlite` (bundled) | |

## How it ingests (on demand, in tiers)

1. **Discovery**: list session files from file metadata plus the head and tail of each file. No index needed.
2. **Open a session**: parse that one file in memory. Nothing is persisted.
3. **Search**: a disposable SQLite FTS5 index, built incrementally in the background. It stores message text plus a pointer (path and byte offset) and per-file progress. Deleting it loses nothing.

## Layout

| Path | Purpose |
| --- | --- |
| `crates/agenteye-core` | Domain types and the `SessionSource` trait. No I/O. |
| `crates/agenteye-sources` | Claude and Codex adapters, JSONL tailing, `Catalog`. |
| `crates/agenteye-index` | SQLite FTS5 search index with incremental sync. |
| `crates/agenteye-cli` | `agenteye` dev CLI: times each tier on your real data. |
| `src-tauri` | Thin Tauri 2 shell; commands only translate to the crates above. |
| `ui` | Angular frontend. |

## Prerequisites

**Windows** (PowerShell):

```powershell
winget install --id Rustlang.Rustup -e          # then reopen the terminal
rustup default stable-x86_64-pc-windows-msvc
rustup component add clippy rustfmt
nvm install 24.15.0; nvm use 24.15.0            # nvm-windows
```

- Visual Studio (or Build Tools) with the **Desktop development with C++** workload (MSVC and Windows SDK)
- WebView2 (included in Windows 11)

**Linux**: Rust via [rustup](https://rustup.rs), Node 24.15+ (for example via nvm), and the WebKitGTK packages from ![https://v2.tauri.app/start/prerequisites/](https://v2.tauri.app/start/prerequisites/).

Check your setup:

```sh
rustc --version   # 1.99 or newer
node --version    # v24.15 or newer
```

## Commands

```sh
npm install && (cd ui && npm install)   # once
npm run dev                              # Tauri app with Angular live reload
cargo fmt --all                          # format
cargo clippy --workspace --all-targets   # lint
cargo test --workspace                   # Rust tests
(cd ui && npm test)                      # UI tests
```

If Chrome is not installed, point the UI tests at Edge:

```powershell
$env:CHROME_BIN="C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
npm test -- --watch=false --browsers=ChromeHeadless
```

Scale spike on your own logs:

```sh
cargo run --release -p agenteye-cli -- discover
cargo run --release -p agenteye-cli -- index --db agenteye.db
cargo run --release -p agenteye-cli -- search --db agenteye.db "some words"
```

Set `CLAUDE_CONFIG_DIR` / `CODEX_HOME` to override the default `~/.claude` / `~/.codex` locations.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or ![http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or ![http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
