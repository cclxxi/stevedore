# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-02

### Added

- Container list with live state, image, ports, and status
- Live per-container stats: CPU %, memory usage/limit, network I/O
- Real-time log viewer with follow mode, scrollback, and top/bottom jumps
- Running-only / all containers filter toggle
- Help overlay with keybindings
- Graceful handling of an unreachable Docker daemon (no panics; terminal restored)

[Unreleased]: https://github.com/cclxxi/stevedore/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/cclxxi/stevedore/releases/tag/v0.1.0
