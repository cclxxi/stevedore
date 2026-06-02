# Contributing to stevedore

Thanks for your interest in improving stevedore! This is a small, focused project,
so contributions of any size — bug reports, docs, or code — are very welcome.

## Development setup

You need a Rust toolchain (1.88 or newer) and a reachable Docker daemon for
manual testing.

```sh
git clone https://github.com/cclxxi/stevedore
cd stevedore
cargo build
cargo run        # launches the TUI against your local Docker daemon
```

## Before opening a pull request

All of these must pass — CI runs the exact same checks:

```sh
cargo fmt --all                          # format
cargo clippy --all-targets -- -D warnings  # lint (warnings are errors)
cargo test                               # unit tests
```

Please also:

- Keep changes focused; one logical change per PR.
- Add or update tests for behavior you change.
- Update `CHANGELOG.md` under the `Unreleased` section.
- Follow [Conventional Commits](https://www.conventionalcommits.org/) for commit
  messages (e.g. `feat: add container restart action`).

## Coding guidelines

- Prefer many small, focused modules over large files.
- Keep functions small and handle errors explicitly (no silent failures).
- Background Docker I/O runs in tokio tasks and communicates with the UI via the
  `AppMessage` channel — never mutate `App` state from a task directly.
- Rendering code in `src/ui/` should be a pure function of `App`; no state changes.

## Architecture overview

- `src/app.rs` — single-owner application state and input handling
- `src/event.rs` — `AppMessage` types sent from tasks to the UI loop
- `src/docker/` — Docker client wrapper, models, stats and log streaming tasks
- `src/ui/` — ratatui rendering (container list, detail, logs, help)
- `src/main.rs` — terminal setup, task spawning, and the main event loop

## Ideas / good first issues

- Container lifecycle actions (start / stop / restart / remove) with confirmation
- Configurable socket path and refresh interval via flags or environment
- Recording a demo GIF (`assets/demo.tape` for [vhs](https://github.com/charmbracelet/vhs))

## License

By contributing, you agree that your contributions will be licensed under the
project's [GPL-3.0-or-later](./LICENSE) license.
