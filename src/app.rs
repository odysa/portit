use std::io::{self, Write};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::{cursor, terminal};

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
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code);
                    }
                }
            }
        }
        Ok(())
    }

    fn setup_display(&mut self, w: &mut impl Write) -> io::Result<()> {
        let (_, term_rows) = terminal::size()?;
        let max_table = ((term_rows as usize) / 2).clamp(3, 20);
        let table_rows = self.filtered_entries.len().min(max_table).max(1);
        self.height = (table_rows + 3).min(term_rows as usize);
        self.visible_rows = self.height.saturating_sub(3);

        // reserve space by scrolling
        for _ in 0..self.height {
            write!(w, "\r\n")?;
        }
        w.flush()?;

        let (_, cur_y) = cursor::position()?;
        self.start_row = (cur_y + 1).saturating_sub(self.height as u16);

        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode) {
        if self.confirm_kill.is_some() {
            self.handle_confirm(code);
            return;
        }

        if self.action_menu.is_some() {
            self.handle_action_menu(code);
            return;
        }

        if self.filter_mode {
            self.handle_filter_input(code);
            return;
        }

        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.next_row(),
            KeyCode::Char('k') | KeyCode::Up => self.prev_row(),
            KeyCode::Enter => self.open_action_menu(),
            KeyCode::Char('/') => self.filter_mode = true,
            KeyCode::Char('K') => self.request_kill(false),
            KeyCode::Char('F') => self.request_kill(true),
            KeyCode::Char('r') => self.refresh(),
            _ => {}
        }
    }

    fn handle_action_menu(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(menu) = &mut self.action_menu {
                    menu.selected = (menu.selected + 1) % ACTIONS.len();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(menu) = &mut self.action_menu {
                    menu.selected = (menu.selected + ACTIONS.len() - 1) % ACTIONS.len();
                }
            }
            KeyCode::Enter => {
                if let Some(menu) = self.action_menu.take() {
                    let force = menu.selected == 1;
                    self.confirm_kill = Some((menu.pid, menu.name));
                    self.confirm_force = force;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.action_menu = None;
            }
            _ => {}
        }
    }

    fn open_action_menu(&mut self) {
        if let Some(&idx) = self.filtered_entries.get(self.selected) {
            let entry = &self.entries[idx];
            self.action_menu = Some(ActionMenu {
                pid: entry.pid,
                name: entry.process_name.clone(),
                selected: 0,
            });
        }
    }

    fn handle_confirm(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some((pid, _)) = self.confirm_kill.take() {
                    ports::kill_process(pid, self.confirm_force);
                    self.confirm_force = false;
                    self.refresh();
                }
            }
            _ => {
                self.confirm_kill = None;
                self.confirm_force = false;
            }
        }
    }

    fn handle_filter_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter => {
                self.filter_mode = false;
                self.apply_filter();
            }
            KeyCode::Esc => {
                self.filter_mode = false;
                self.filter.clear();
                self.apply_filter();
            }
            KeyCode::Backspace => {
                self.filter.pop();
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
            }
            _ => {}
        }
    }

    fn next_row(&mut self) {
        if self.filtered_entries.is_empty() {
            return;
        }
        self.selected = if self.selected >= self.filtered_entries.len() - 1 {
            0
        } else {
            self.selected + 1
        };
        self.ensure_visible();
    }

    fn prev_row(&mut self) {
        if self.filtered_entries.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.filtered_entries.len() - 1
        } else {
            self.selected - 1
        };
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        if self.visible_rows == 0 {
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected - self.visible_rows + 1;
        }
    }

    fn request_kill(&mut self, force: bool) {
        if let Some(&idx) = self.filtered_entries.get(self.selected) {
            let entry = &self.entries[idx];
            self.confirm_kill = Some((entry.pid, entry.process_name.clone()));
            self.confirm_force = force;
        }
    }

    fn refresh(&mut self) {
        self.entries = ports::list_listening_ports();
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        let query = self.filter.to_lowercase();
        self.filtered_entries = if query.is_empty() {
            (0..self.entries.len()).collect()
        } else {
            self.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.process_name.to_lowercase().contains(&query)
                        || e.port.to_string().contains(&query)
                })
                .map(|(i, _)| i)
                .collect()
        };

        if self.filtered_entries.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.filtered_entries.len() - 1);
        }
        self.scroll_offset = 0;
    }
}
