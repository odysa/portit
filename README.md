# portit

A minimal TUI for inspecting listening TCP ports and killing processes.

**~379KB** release binary. Single dependency (`crossterm`). No runtime overhead.

![demo](static/demo.gif)

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/odysa/portit/main/install.sh | sh
```

Or with Cargo:

```bash
cargo install portit
```

## Usage

Run `portit` to see all listening TCP ports.

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `Enter` | Open action menu |
| `/` | Filter by name or port |
| `K` | Kill (SIGTERM) |
| `F` | Force kill (SIGKILL) |
| `r` | Refresh |
| `q` | Quit |

## Requirements

- macOS or Linux
- `lsof` and `kill` in PATH

## License

MIT
