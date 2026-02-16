mod app;
mod ports;
mod ui;

use std::io;

use crossterm::{cursor, execute, terminal};

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide)?;

    let mut app = app::App::new();
    let result = app.run(&mut stdout);

    // move cursor below our area so shell prompt is clean
    if app.height > 0 {
        let _ = execute!(stdout, cursor::MoveTo(0, app.start_row + app.height as u16));
    }
    let _ = execute!(stdout, cursor::Show);
    let _ = terminal::disable_raw_mode();
    println!();

    result
}
