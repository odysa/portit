mod input;
mod state;

use std::io::{self, Write};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::{cursor, execute, terminal};

use crate::ports::{self, PortEntry};
use crate::ui;

pub const ACTIONS: [&str; 2] = ["Kill (SIGTERM)", "Force Kill (SIGKILL)"];

pub struct ActionMenu {
    pub pid: u32,
    pub name: String,
    pub selected: usize,
}

pub struct App {
    pub entries: Vec<PortEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub filter_mode: bool,
    pub filtered_entries: Vec<usize>,
    pub should_quit: bool,
    pub confirm_kill: Option<(u32, String)>,
    pub confirm_force: bool,
    pub action_menu: Option<ActionMenu>,
    pub status_msg: Option<String>,
    pub start_row: u16,
    pub height: usize,
    pub visible_rows: usize,
}

impl App {
    pub fn new() -> Self {
        let entries = ports::list_listening_ports();
        let filtered_entries: Vec<usize> = (0..entries.len()).collect();
        Self {
            entries,
            selected: 0,
            scroll_offset: 0,
            filter: String::new(),
            filter_mode: false,
            filtered_entries,
            should_quit: false,
            confirm_kill: None,
            confirm_force: false,
            action_menu: None,
            status_msg: None,
            start_row: 0,
            height: 0,
            visible_rows: 0,
        }
    }

    pub fn run(&mut self, w: &mut impl Write) -> io::Result<()> {
        self.setup_display(w)?;

        while !self.should_quit {
            ui::render(w, self)?;
            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key(key.code);
                    }
                    Event::Resize(_, h) => self.recalc_layout(h as usize),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn setup_display(&mut self, w: &mut impl Write) -> io::Result<()> {
        let (_, term_rows) = terminal::size()?;
        self.recalc_layout(term_rows as usize);

        for _ in 0..self.height {
            write!(w, "\r\n")?;
        }
        w.flush()?;

        let (_, cur_y) = cursor::position()?;
        self.start_row = (cur_y + 1).saturating_sub(self.height as u16);

        Ok(())
    }

    fn recalc_layout(&mut self, term_rows: usize) {
        let max_table = (term_rows / 2).clamp(3, 20);
        let table_rows = self.filtered_entries.len().min(max_table).max(1);
        self.height = (table_rows + 3).min(term_rows);
        self.visible_rows = self.height.saturating_sub(3);
    }

    pub fn install_panic_hook() {
        let default = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = terminal::disable_raw_mode();
            let _ = execute!(io::stdout(), cursor::Show);
            default(info);
        }));
    }
}

fn cycle_index(current: usize, len: usize, step: isize) -> usize {
    if len == 0 {
        return 0;
    }
    ((current as isize + step).rem_euclid(len as isize)) as usize
}
