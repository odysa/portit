use crate::ports::{self, PortEntry};

use super::{App, cycle_index};

impl App {
    pub(super) fn next_row(&mut self) {
        self.move_selection(1);
    }

    pub(super) fn prev_row(&mut self) {
        self.move_selection(-1);
    }

    fn move_selection(&mut self, step: isize) {
        if self.filtered_entries.is_empty() {
            return;
        }
        self.selected = cycle_index(self.selected, self.filtered_entries.len(), step);
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

    pub(super) fn request_kill(&mut self, force: bool) {
        if let Some(entry) = self.selected_entry() {
            self.confirm_kill = Some((entry.pid, entry.process_name.clone()));
            self.confirm_force = force;
        }
    }

    pub(super) fn refresh(&mut self) {
        self.entries = ports::list_listening_ports();
        self.apply_filter();
    }

    pub(super) fn apply_filter(&mut self) {
        let query = self.filter.to_ascii_lowercase();
        self.filtered_entries = if query.is_empty() {
            (0..self.entries.len()).collect()
        } else {
            self.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.process_name.to_ascii_lowercase().contains(&query)
                        || e.port.to_string().contains(&query)
                        || e.command.to_ascii_lowercase().contains(&query)
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

    pub(super) fn selected_entry(&self) -> Option<&PortEntry> {
        let idx = self.filtered_entries.get(self.selected)?;
        self.entries.get(*idx)
    }
}
