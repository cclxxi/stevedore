# stevedore

[![CI](https://github.com/cclxxi/stevedore/actions/workflows/ci.yml/badge.svg)](https://github.com/cclxxi/stevedore/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/stevedore.svg)](https://crates.io/crates/stevedore)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)

A super-lightweight terminal UI for monitoring Docker containers and their logs —
a tiny, read-only alternative to Portainer that runs straight in your console.

No web server, no agent, no database. Just a single **~1 MB** binary that talks to
the local Docker socket.

<!-- Absolute URL so the demo renders on both GitHub and crates.io.
     Regenerate with: vhs assets/demo.tape -->
![demo](https://raw.githubusercontent.com/cclxxi/stevedore/master/assets/demo.gif)

## Why stevedore

- **Tiny** — a single statically-optimized binary, ~1 MB, near-zero idle footprint.
- **Zero setup** — no daemon, no agent, no config. Run it where your containers live.
- **Read-only & safe** — it monitors; it doesn't (yet) start/stop anything.
- **SSH-friendly** — perfect for quickly checking containers on a remote server.

## Features

- **Container list** with live state, image, ports and status
- **Live stats** — per-container CPU %, memory usage/limit, network I/O
- **Log viewer** — follow logs in real time, scroll back, jump to top/bottom
- **Running / all** filter toggle
- Graceful errors when the daemon is unreachable (no panics, terminal always restored)

## Requirements

- A reachable Docker daemon (default socket `/var/run/docker.sock`)
- Your user must be able to access the socket (the `docker` group, or run with `sudo`)
- Rust 1.88+ (only when building from source)

## Install

### Homebrew (macOS / Linux)

```sh
brew install cclxxi/tap/stevedore
```

### Install script (no Rust needed)

Installs the latest release binary for your platform:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/cclxxi/stevedore/releases/latest/download/stevedore-installer.sh | sh
```

### From crates.io

```sh
cargo install stevedore
```

### Prebuilt binaries

Download a tarball for your platform from the
[Releases](https://github.com/cclxxi/stevedore/releases) page.

### From source

```sh
git clone https://github.com/cclxxi/stevedore
cd stevedore
cargo install --path .
```

## Usage

```sh
stevedore
```

### Keybindings

| Key | Action |
|---|---|
| `↑`/`k`, `↓`/`j` | Move selection / scroll logs |
| `g` / `G` | Jump to top / bottom |
| `a` | Toggle all / running-only |
| `Enter` / `l` | Open logs for the selected container |
| `PgUp` / `PgDn` | Page through logs |
| `f` | Toggle log follow |
| `Esc` / `q` | Back to list (in logs) / quit (in list) |
| `?` | Toggle help |
| `q` | Quit |

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md) for how to set
up, run the checks, and open a pull request. Good first issues are labeled
[`good first issue`](https://github.com/cclxxi/stevedore/labels/good%20first%20issue).

## License

Licensed under the [GNU General Public License v3.0 or later](./LICENSE).

Copyright © 2026 Ilia Proshin.
