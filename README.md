# portit

`portit` is a small terminal UI for listing listening TCP ports and killing the owning process.

**~379KB** release binary. Single dependency (`crossterm`). No runtime overhead.

![demo](static/demo.gif)

## Features

- Lists listening TCP ports from `lsof`
- Shows PID, process name, address, and port
- Filter by process name or port (`/`)
- Kill selected process with `SIGTERM` or `SIGKILL`

## Requirements

- macOS or Linux
- `lsof` available in `PATH`
- `kill` available in `PATH`

## Run from source

```bash
cargo run --release
```

## Keybindings

- `q` or `Esc`: quit
- `j` / `k` or arrow keys: move selection
- `Enter`: open action menu
- `/`: filter mode
- `K`: kill (`SIGTERM`)
- `F`: force kill (`SIGKILL`)
- `r`: refresh

## Install

### Cargo (after publishing to crates.io)

```bash
cargo install portit
```

### Homebrew (after creating a tap)

```bash
brew tap <you>/tap
brew install portit
```

## Publish to crates.io

1. Add package metadata in `Cargo.toml`:
   - `description`
   - `license`
   - `repository`
   - `readme = "README.md"`
2. Run:

```bash
cargo test
cargo package
cargo publish --dry-run
```

3. Login and publish:

```bash
cargo login <CRATES_IO_TOKEN>
cargo publish
```

## Publish to Homebrew (tap formula)

1. Create a Git tag and GitHub release (example: `v0.1.0`)
2. In your tap repo, add `Formula/portit.rb`
3. Use release tarball URL and SHA256 in formula
4. Test locally:

```bash
brew install --build-from-source ./Formula/portit.rb
brew test portit
```

## License

Add a `LICENSE` file (for example MIT) before publishing.
