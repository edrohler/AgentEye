# AGENTS.md

Guidance for AI coding agents (and humans) working in AgentEye. `CLAUDE.md` imports this file, so this is the single source of truth.

## Project

AgentEye is a read-only desktop app for browsing, searching and monitoring AI agent sessions (Claude Code and Codex CLI in v1). Rust core, Tauri 2 shell, Angular UI. Windows and Linux.

Read [README.md](README.md) for layout and commands and [docs/IDEATION.md](docs/IDEATION.md) for decisions already made. Do not re-litigate those decisions; propose changes explicitly instead.

Core rules that shape every change:

- **Read-only.** Never write to, lock, or delete agent session files. Open them read-only and release them immediately.
- **On-demand ingestion in tiers** (discovery, open-in-memory, background search index). Session content is never copied into storage except as the rebuildable search index. Store paths and byte offsets, not sessions.
- **Local only.** No network calls, no telemetry. Session logs contain code and secrets.

## About the maintainer: teach Rust as you go

The maintainer last wrote C++ around 2013 and is using this project to learn Rust. When you write or change Rust:

- Prefer **simple, idiomatic, readable** code over clever code. If two approaches are idiomatic, choose the simpler and say why in one sentence.
- The first time a concept appears in a change, explain it briefly with a C++ comparison, then move on. Do not repeat explanations for concepts already covered. Examples: ownership and moves (vs RAII and `std::move`), borrowing and lifetimes (vs references and dangling pointers), `Result` and `?` (vs exceptions), `enum` with data (vs `union`/`std::variant`), traits (vs abstract classes and templates/concepts), `Option` (vs null), iterators (vs loops and `<algorithm>`).
- Put explanations in your reply, not as noisy code comments. Code comments explain *why*, never *what*.
- Point to the relevant chapter of *The Rust Programming Language* (https://doc.rust-lang.org/book/) when a topic deserves a deeper read.
- When the compiler or clippy complains, explain what the message means before fixing it.

## Architecture and design principles

### SOLID, applied to this codebase

- **Single responsibility.** One reason to change per crate, module, struct, component and service. Crates: `core` (types and contracts, no I/O), `sources` (agent adapters), `index` (search cache), `cli` (dev tool), `src-tauri` (thin shell), `ui` (presentation).
- **Open/closed.** Add an agent by implementing `SessionSource` and registering it in `Catalog`; do not edit existing adapters or add `match agent { ... }` branches in shared code.
- **Liskov substitution.** Any `SessionSource` must honor the trait's documented contract (missing root is empty not an error, only complete lines consumed, bad lines skipped). Add a shared contract test when a new adapter is added.
- **Interface segregation.** Keep traits small and focused. Do not grow `SessionSource` into a catch-all; add a separate trait when a capability is optional.
- **Dependency inversion.** Depend on traits, not concrete types, at crate boundaries. Dependency direction is `core` <- `sources`/`index` <- `cli`/`src-tauri`. `core` depends on nothing internal. Never import upward.

### DRY

- Remove duplicated *knowledge* (a format rule, a constant, a mapping), not merely similar-looking code. Wait for the third occurrence before abstracting; a wrong abstraction costs more than duplication.
- Shared parsing and filesystem logic lives in `agenteye-sources` helpers (`jsonl`, `fs`, `content`, `text`). Check there before writing another.
- The Rust-to-TypeScript contract is the one place types are mirrored by hand (`SessionSummary` etc.). Keep the TypeScript in `ui/src/app/core/models.ts` and the serde attributes in `agenteye-core` in lockstep, and change both in the same commit.

### Maintainability

- Small functions and modules; one concept per file. Prefer composition over inheritance-like patterns.
- Make invalid states unrepresentable (enums, newtypes) instead of validating strings and flags at runtime.
- Public items get doc comments (`///`) stating the contract and edge cases. No dead code, no commented-out code, no TODOs without a tracked reason.
- No new dependency without a reason. Declare versions once in the root `[workspace.dependencies]` and reference them with `.workspace = true`. Prefer well-maintained, widely used crates. Ask before adding one.
- Do not refactor unrelated code or expand scope. If a larger refactor looks warranted, explain it first.

## Rust guidelines

- **Edition and toolchain:** stable Rust via `rust-toolchain.toml`; `rust-version` in the workspace is the declared minimum. Format with `cargo fmt`, lint with `cargo clippy --workspace --all-targets` (pedantic is on and must stay warning-free). `unsafe` is forbidden workspace-wide.
- **Errors:** libraries return `Result<T, E>` with a `thiserror` enum that has useful context (for example the path). Binaries may use `anyhow` with `.with_context(...)`. Propagate with `?`. Do not use `unwrap()` or `expect()` in non-test code except for a documented invariant that cannot fail. Never `panic!` on bad input from disk or the frontend.
- **Ownership:** borrow (`&str`, `&[u8]`, `&Path`) in parameters; own (`String`, `PathBuf`) in stored data. Do not `clone()` to silence the borrow checker; restructure first. Accept `impl AsRef<Path>` or `impl Into<PathBuf>` at public boundaries.
- **Types:** use enums for closed sets, newtypes for IDs and units, `Option` for absence. Derive standard traits (`Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`) where meaningful, and `Serialize`/`Deserialize` only on types that cross a boundary.
- **Iterators over index loops**, `?` over nested `match`, `let ... else` for early returns, `matches!` for simple pattern checks.
- **Parsing log files:** be tolerant (skip unknown or malformed lines, ignore unknown fields), stream rather than load whole files, never assume a line fits a small buffer (single lines can be megabytes), and never consume a trailing line without a newline (it may be mid-write).
- **Concurrency:** `rayon` for CPU-bound fan-out, a single writer for SQLite. Never do blocking file or database work on the async runtime; use `spawn_blocking`.
- **Tauri:** commands are thin adapters (validate input, call a crate, map errors to `String`). No domain logic in `src-tauri`. Any path received from the frontend is untrusted; validate it against discovered sessions before reading. Grant the minimum capabilities and keep the CSP strict.
- **Tests:** unit tests in a `#[cfg(test)] mod tests` beside the code, integration tests in the crate's `tests/` directory, `tempfile` for filesystem fixtures, no dependence on the developer's real `~/.claude` or `~/.codex`. A bug fix starts with a failing test.

## Angular guidelines (Angular 22, signals, zoneless)

- **Standalone only.** No NgModules. Zoneless change detection (`provideZonelessChangeDetection`) is already configured. Do not add `zone.js`.
- **Signals first.** Component and service state is `signal()`; derived state is `computed()`; use `linkedSignal()` for state that resets from another signal. Use `effect()` only for side effects that cannot be expressed as derivation (logging, syncing to non-Angular APIs), never to set other signals. Expose state as readonly (`asReadonly()`).
- **Component API:** `input()` / `input.required()`, `output()`, `model()`, `viewChild()` / `contentChild()` signal queries. Do not use the decorator forms (`@Input`, `@Output`, `@ViewChild`).
- **Change detection:** every component sets `changeDetection: ChangeDetectionStrategy.OnPush`.
- **Templates:** built-in control flow (`@if`, `@for` with a stable `track`, `@switch`, `@defer`). No `*ngIf`/`*ngFor`. Use `[class.x]` and `[style.x]` bindings, not `ngClass`/`ngStyle`. Keep templates declarative; move logic into `computed()` signals, not template method calls that do work.
- **Dependency injection:** `inject()` rather than constructor parameters. Services are `providedIn: 'root'` unless intentionally scoped, with a single responsibility.
- **Host bindings:** use the component `host` metadata, not `@HostBinding`/`@HostListener`.
- **Async data:** model loading, error and value as signals (or Angular's `resource`/`httpResource` where it fits). Never subscribe manually in components when a signal or `toSignal()` works; if RxJS is needed, clean up with `takeUntilDestroyed()`.
- **Tauri boundary:** all `invoke(...)` calls live in services under `core/`, never in components. Components depend on the service, which makes them testable by faking the service. Backend types are mirrored in `core/models.ts`.
- **Structure:** feature folders under `app/features/<feature>/`, shared singletons in `app/core/`, reusable presentational pieces in `app/shared/`. File names follow the current style guide (`session-list.ts`, `session-list.html`, `session-list.scss`; no `.component` suffix). Lazy-load routes when routing is introduced.
- **TypeScript:** `strict` stays on. No `any`; use `unknown` and narrow. Prefer `readonly` and immutable updates to signals (`set`/`update` with new values, never mutate in place).
- **Large lists:** session and message lists can be huge, so never render an unbounded list: the DOM must stay bounded however much data exists. Prefer bounded pagination ("Showing X of Y" plus a "View more" button, with the batch size as a setting), as the session list does. It adds no dependency, and every rendered row is a real list item, so screen readers, find-in-page and keyboard navigation work normally. Reach for virtual scrolling only if pagination demonstrably cannot work, and say why.
- **Styling (Tailwind CSS v4):** style with utility classes in templates; there is no component SCSS and no Sass (Tailwind v4 does not support preprocessors). Colors come from the semantic tokens in `ui/src/styles.css` (`bg-surface`, `text-fg`, `text-muted`, `border-line`, `text-accent`, `text-danger`); never hard-code a color in a template, and never add `dark:` variants, because dark mode is one override block in `styles.css`. When the same utility combination repeats across components, extract it once as a class in `@layer components` (see `.btn`, `.agent-label`) instead of copying it (DRY). Layout of a component's host element goes in `host: { class: '...' }`, not a wrapper `div`.
- **Accessibility:** semantic elements, keyboard-operable controls, labels on icon-only buttons, visible focus styles. Tests select by role, text or `data-testid`, never by styling classes.
- **Tests:** a spec beside each component or service. Fake `SessionService` rather than mocking `@tauri-apps/api`. Test behavior (what the user sees), not implementation. Use `provideZonelessChangeDetection()` in TestBed.
- When unsure about an Angular API in this version, check https://angular.dev rather than relying on memory; the framework moves quickly.

## Commands

```sh
cargo fmt --all                          # format
cargo clippy --workspace --all-targets   # lint (must be clean)
cargo test --workspace                   # Rust tests
(cd ui && npm test)                      # UI tests (set CHROME_BIN to Edge if Chrome is missing)
(cd ui && npm run build)                 # UI production build
npm run dev                              # run the Tauri app
```

## Definition of done

1. Code builds, `cargo clippy` and `cargo fmt --check` are clean, and relevant Rust and UI tests pass. Report what you actually ran; do not claim success for something you did not run.
2. New behavior has tests; bug fixes have a regression test.
3. Public items are documented; README or docs are updated when commands, layout or decisions change.
4. No unrelated changes in the diff.

## Git

Do not stage, commit, push, branch or tag unless explicitly asked. When work is complete, provide a suggested commit message (Conventional Commits, for example `feat(index): ...`). Do not add co-author trailers.
