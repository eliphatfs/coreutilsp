# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Parallel implementations of GNU coreutils in Rust. Binaries: `du-par` (parallel du) and `rm-par` (parallel rm). Uses rayon/crossbeam for parallelism with minimal memory footprint.

## Build & Test Commands

```bash
cargo build -r              # Release build, binaries in ./target/release
cargo build                 # Debug build
cargo test                  # Run all tests
cargo test test_name        # Run a single test
```

Control parallelism via `RAYON_NUM_THREADS` env var.

## Architecture

- `src/bin/du-par.rs` — parallel `du` binary
- `src/bin/rm-par.rs` — parallel `rm` binary
- `src/utils/` — shared utilities:
  - `work_entry.rs` — work queue entry abstraction
  - `size_unit.rs` — size parsing/formatting (human-readable, thresholds)
  - `clap_ext.rs` — CLI argument parsing extensions
- `tests/` — integration tests

Key deps: `rayon` (thread pool), `crossbeam-deque` (work stealing), `clap` (CLI), `nom` (parsing), `filesize` (formatting).

Edition: Rust 2024.
