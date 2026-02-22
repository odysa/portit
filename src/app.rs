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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::PortEntry;

    fn test_app(n: usize) -> App {
        let entries: Vec<PortEntry> = (0..n)
            .map(|i| PortEntry {
                pid: 1000 + i as u32,
                process_name: format!("proc{}", i),
                port: 3000 + i as u16,
                address: "127.0.0.1".to_string(),
            })
            .collect();
        let filtered_entries: Vec<usize> = (0..n).collect();
        App {
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
            visible_rows: 5,
        }
    }

    // cycle_index tests

    #[test]
    fn cycle_forward() {
        assert_eq!(cycle_index(0, 5, 1), 1);
        assert_eq!(cycle_index(4, 5, 1), 0); // wraps
    }

    #[test]
    fn cycle_backward() {
        assert_eq!(cycle_index(2, 5, -1), 1);
        assert_eq!(cycle_index(0, 5, -1), 4); // wraps
    }

    #[test]
    fn cycle_empty() {
        assert_eq!(cycle_index(0, 0, 1), 0);
        assert_eq!(cycle_index(0, 0, -1), 0);
    }

    #[test]
    fn cycle_single() {
        assert_eq!(cycle_index(0, 1, 1), 0);
        assert_eq!(cycle_index(0, 1, -1), 0);
    }

    // recalc_layout tests

    #[test]
    fn layout_small_terminal() {
        let mut app = test_app(10);
        app.recalc_layout(10);
        // max_table = (10/2).clamp(3,20) = 5
        // table_rows = min(10, 5).max(1) = 5
        // height = min(5+3, 10) = 8
        // visible = 8 - 3 = 5
        assert_eq!(app.height, 8);
        assert_eq!(app.visible_rows, 5);
    }

    #[test]
    fn layout_large_terminal() {
        let mut app = test_app(10);
        app.recalc_layout(50);
        // max_table = (50/2).clamp(3,20) = 20
        // table_rows = min(10, 20).max(1) = 10
        // height = min(10+3, 50) = 13
        // visible = 13 - 3 = 10
        assert_eq!(app.height, 13);
        assert_eq!(app.visible_rows, 10);
    }

    #[test]
    fn layout_tiny_terminal() {
        let mut app = test_app(10);
        app.recalc_layout(4);
        // max_table = (4/2).clamp(3,20) = 3
        // table_rows = min(10, 3).max(1) = 3
        // height = min(3+3, 4) = 4
        // visible = 4 - 3 = 1
        assert_eq!(app.height, 4);
        assert_eq!(app.visible_rows, 1);
    }

    #[test]
    fn layout_no_entries() {
        let mut app = test_app(0);
        app.recalc_layout(20);
        // max_table = (20/2).clamp(3,20) = 10
        // table_rows = min(0, 10).max(1) = 1
        // height = min(1+3, 20) = 4
        // visible = 4 - 3 = 1
        assert_eq!(app.height, 4);
        assert_eq!(app.visible_rows, 1);
    }

    // selection & scrolling tests

    #[test]
    fn next_row_moves_forward() {
        let mut app = test_app(5);
        app.next_row();
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn next_row_wraps() {
        let mut app = test_app(3);
        app.selected = 2;
        app.next_row();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn prev_row_wraps() {
        let mut app = test_app(3);
        app.prev_row();
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn scroll_follows_selection_down() {
        let mut app = test_app(10);
        app.visible_rows = 3;
        for _ in 0..5 {
            app.next_row();
        }
        assert_eq!(app.selected, 5);
        assert!(app.scroll_offset + app.visible_rows > app.selected);
    }

    #[test]
    fn scroll_follows_selection_up() {
        let mut app = test_app(10);
        app.visible_rows = 3;
        app.selected = 5;
        app.scroll_offset = 5;
        app.prev_row();
        assert_eq!(app.selected, 4);
        assert!(app.scroll_offset <= app.selected);
    }

    #[test]
    fn empty_list_selection() {
        let mut app = test_app(0);
        app.next_row(); // should not panic
        app.prev_row(); // should not panic
        assert_eq!(app.selected, 0);
    }

    // filter tests

    #[test]
    fn filter_by_name() {
        let mut app = test_app(5);
        app.filter = "proc2".to_string();
        app.apply_filter();
        assert_eq!(app.filtered_entries, vec![2]);
    }

    #[test]
    fn filter_case_insensitive() {
        let mut app = test_app(5);
        app.filter = "PROC1".to_string();
        app.apply_filter();
        assert_eq!(app.filtered_entries, vec![1]);
    }

    #[test]
    fn filter_by_port() {
        let mut app = test_app(5);
        app.filter = "3002".to_string();
        app.apply_filter();
        assert_eq!(app.filtered_entries, vec![2]);
    }

    #[test]
    fn filter_empty_shows_all() {
        let mut app = test_app(5);
        app.filter = String::new();
        app.apply_filter();
        assert_eq!(app.filtered_entries, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn filter_no_match() {
        let mut app = test_app(5);
        app.filter = "zzzzz".to_string();
        app.apply_filter();
        assert!(app.filtered_entries.is_empty());
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn filter_clamps_selection() {
        let mut app = test_app(5);
        app.selected = 4;
        app.filter = "proc0".to_string();
        app.apply_filter();
        assert_eq!(app.selected, 0);
    }

    // selected_entry tests

    #[test]
    fn selected_entry_valid() {
        let app = test_app(3);
        let entry = app.selected_entry().unwrap();
        assert_eq!(entry.pid, 1000);
    }

    #[test]
    fn selected_entry_empty() {
        let mut app = test_app(0);
        app.filtered_entries.clear();
        assert!(app.selected_entry().is_none());
    }
}
