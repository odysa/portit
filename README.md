# portit

`portit` is a terminal UI for viewing listening TCP ports and stopping the process that owns them.

## What it does

- Reads listening sockets with `lsof -iTCP -sTCP:LISTEN -P -n`
- Shows `PID`, process name, protocol, address, and port
- Lets you filter by process name or port substring
- Sends `SIGTERM` or `SIGKILL` to the selected PID

## Requirements

- macOS or Linux
- `lsof` in `PATH`
- `kill` in `PATH`
- Terminal width of at least 40 columns

## Quick start

```bash
cargo run --release
```

`portit` opens in raw terminal mode and restores your terminal when you quit.

## Controls

### Main view

- `q` or `Esc`: quit
- `j` / `Down`: next row
- `k` / `Up`: previous row
- `Enter`: open action menu
- `/`: enter filter mode
- `K`: request `SIGTERM` for selected process
- `F`: request `SIGKILL` for selected process
- `r`: refresh from `lsof`

### Action menu

- `j` / `Down`: next action
- `k` / `Up`: previous action
- `Enter`: confirm selected action
- `Esc` or `q`: close menu

### Kill confirmation popup

- `y` or `Y`: send signal
- Any other key: cancel

### Filter mode

- Type: edit query
- `Backspace`: delete character
- `Enter`: apply filter
- `Esc`: clear filter and return

## Filtering behavior

- Case-insensitive match on process name
- String match on port number (for example, `30` matches `3000` and `8080`)
- Filtering is applied when you press `Enter` in filter mode

## Manual

For full operational details, see `MANUAL.md`.

## Development

```bash
cargo test
```

## Packaging notes

- Crates.io/Homebrew sections were intentionally removed from this quick-start README.
- Keep release and distribution docs in a separate release guide if needed.
