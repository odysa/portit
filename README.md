portit
======

[![CI](https://github.com/odysa/portit/actions/workflows/ci.yml/badge.svg)](https://github.com/odysa/portit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/portit)](https://crates.io/crates/portit)
[![Downloads](https://img.shields.io/crates/d/portit)](https://crates.io/crates/portit)
[![License](https://img.shields.io/crates/l/portit)](https://github.com/odysa/portit/blob/main/LICENSE)

A minimal Rust TUI for inspecting listening TCP ports and killing processes.

**~168KB** release binary. Single dependency (`crossterm`). No runtime overhead.

![demo](static/demo.gif)

## Features

- List all listening TCP ports with PID, process name, protocol, address, and port
- Real-time filtering by process name or port number (case-insensitive)
- Kill processes with SIGTERM or SIGKILL with confirmation
- Vim-style navigation (j/k or arrow keys)
- Wrapping selection and scrolling for long lists
- Inline display without taking over the full terminal
- Tiny release binary with aggressive optimization
- Clean terminal restoration on exit

## Installation

### Homebrew
```bash
brew install odysa/tap/portit
```

### Using install script
```bash
curl -fsSL https://raw.githubusercontent.com/odysa/portit/main/install.sh | sh
```

### Using cargo
```bash
cargo install portit
```

### Build from source
```bash
git clone https://github.com/odysa/portit.git
cd portit
cargo build --release
```

## Usage

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `k` or arrow keys | Move selection up/down |
| `Enter` | Open action menu |
| `/` | Filter by name or port |
| `K` | Kill selected process (SIGTERM) |
| `F` | Force kill selected process (SIGKILL) |
| `r` | Refresh port list |
| `q` / `Esc` | Quit |

The action menu also supports `j`/`k` for navigation, `Enter` to select, and `Esc` to close.

## Requirements

- macOS or Linux (x86_64 or aarch64)
- `lsof` and `kill` in PATH

## License

MIT. See [LICENSE](LICENSE) for details.
