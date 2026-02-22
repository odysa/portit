use crossterm::event::KeyCode;

use crate::ports;

use super::{ACTIONS, ActionMenu, App, cycle_index};

impl App {
    pub(super) fn handle_key(&mut self, code: KeyCode) {
        self.status_msg = None;

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
            KeyCode::Char('j') | KeyCode::Down => self.move_action_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_action_selection(-1),
            KeyCode::Enter => {
                if let Some(menu) = self.action_menu.take() {
                    self.confirm_kill = Some((menu.pid, menu.name));
                    self.confirm_force = menu.selected == 1;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.action_menu = None;
            }
            _ => {}
        }
    }

    fn move_action_selection(&mut self, step: isize) {
        if let Some(menu) = &mut self.action_menu {
            menu.selected = cycle_index(menu.selected, ACTIONS.len(), step);
        }
    }

    fn open_action_menu(&mut self) {
        if let Some(entry) = self.selected_entry() {
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
                if let Some((pid, name)) = self.confirm_kill.take() {
                    if ports::kill_process(pid, self.confirm_force) {
                        self.status_msg = Some(format!("Killed {} (PID {})", name, pid));
                    } else {
                        self.status_msg =
                            Some(format!("Failed to kill {} (PID {})", name, pid));
                    }
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
}
