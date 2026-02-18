# portit Manual

## 1. Purpose

`portit` is an interactive terminal tool to inspect listening TCP ports and stop the owning process by PID.

## 2. Data source

On startup and refresh, `portit` runs:

```bash
lsof -iTCP -sTCP:LISTEN -P -n
```

The result is parsed into table entries with:

- process name
- PID
- address
- port

Entries are sorted by port and deduplicated by `(PID, port)`.

## 3. Starting and quitting

Start from source:

```bash
cargo run --release
```

Quit with `q` or `Esc` from the main view.

## 4. Screen layout

The UI has:

- title/status row
- column header row
- scrollable process rows
- footer help row

If terminal width is below 40 columns, `portit` does not render the table.

## 5. Navigation model

In the main view:

- `j` or `Down` moves selection down
- `k` or `Up` moves selection up

Selection wraps at both ends of the filtered list.

## 6. Action flow

### Open action menu

Press `Enter` on a selected row to open a 2-item menu:

1. `Kill (SIGTERM)`
2. `Force Kill (SIGKILL)`

### Choose action

- `j` / `Down`: next item
- `k` / `Up`: previous item
- `Enter`: move to confirmation
- `Esc` or `q`: close menu

### Confirm action

In confirmation popup:

- `y` or `Y`: send signal with `kill`
- any other key: cancel

After successful confirmation, the list is refreshed.

## 7. Direct kill shortcuts

From main view:

- `K` opens confirmation for `SIGTERM`
- `F` opens confirmation for `SIGKILL`

These skip the action menu and go directly to confirmation.

## 8. Filtering

Enter filter mode with `/`.

In filter mode:

- type characters to build query
- `Backspace` deletes last character
- `Enter` applies filter
- `Esc` clears filter and exits filter mode

Filter behavior:

- process name: ASCII case-insensitive substring match
- port: string contains match

Examples:

- `node` matches `Node`, `node`, `NODE`
- `30` matches `3000`, `3030`, `13000`

## 9. Refresh behavior

Press `r` to repopulate rows from `lsof` and re-apply current filter.

## 10. Signals and permissions

`portit` delegates to system `kill`:

- `SIGTERM` for graceful stop
- `SIGKILL` for force stop

If you do not own the process (or lack privileges), signal delivery may fail.

## 11. Failure behavior

- If `lsof` execution fails, the list is empty.
- If `kill` fails, no explicit error dialog is shown; the table refreshes after confirmed actions.

## 12. Troubleshooting

### No rows shown

- confirm something is listening: `lsof -iTCP -sTCP:LISTEN -P -n`
- widen terminal to at least 40 columns
- clear filter with `/` then `Esc`

### Could not stop process

- check ownership/permissions for target PID
- retry with force kill (`F` or action menu option 2)
- verify process still exists with `ps -p <PID>`

## 13. Command reference

| Context | Keys | Effect |
| --- | --- | --- |
| Main | `q`, `Esc` | Quit |
| Main | `j`, `Down` | Next row |
| Main | `k`, `Up` | Previous row |
| Main | `Enter` | Open action menu |
| Main | `/` | Enter filter mode |
| Main | `K` | Confirm `SIGTERM` |
| Main | `F` | Confirm `SIGKILL` |
| Main | `r` | Refresh list |
| Action menu | `j`, `Down` | Next action |
| Action menu | `k`, `Up` | Previous action |
| Action menu | `Enter` | Select action |
| Action menu | `Esc`, `q` | Close action menu |
| Confirm popup | `y`, `Y` | Execute kill |
| Confirm popup | any other key | Cancel |
| Filter mode | text input | Edit query |
| Filter mode | `Backspace` | Delete character |
| Filter mode | `Enter` | Apply filter |
| Filter mode | `Esc` | Clear filter and exit |
