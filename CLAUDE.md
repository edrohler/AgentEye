@AGENTS.md

## Claude Code specifics

- The project guidance above lives in `AGENTS.md`; edit it there, not here.
- This is a learning project for Rust. Follow the "teach Rust as you go" section in every Rust change.
- Prefer targeted Grep/Glob/Read over broad exploration, and read only the crates or components relevant to the task.
- Run `cargo clippy --workspace --all-targets` and the relevant tests after Rust changes, and `npm test` in `ui` after UI changes, before reporting done.
