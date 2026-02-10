## fswhy

This project is an early-stage experiment toward building a TUI-based
disk usage analyzer in Rust.

Currently, it implements a recursive filesystem scanner and a stable
internal data model. Interactive features and TUI are planned but not
yet implemented.

## Current Features

- Recursive filesystem scanning
- Unified node model for files and directories
- Cross-platform (Windows / Linux / macOS)

## Usage

```bash
cargo run -- <path>
```

## Roadmap

- [ ] Logical folding and node indexing
- [ ] Interactive navigation
- [ ] TUI interface
- [ ] Performance optimization for large directories