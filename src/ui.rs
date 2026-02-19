use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{self, Clear, ClearType},
};

use crate::app::{ACTIONS, App};

const PID_W: usize = 8;
const PROTO_W: usize = 6;
const ADDR_W: usize = 18;
const PORT_W: usize = 6;
const FIXED_W: usize = PID_W + PROTO_W + ADDR_W + PORT_W + 5;
const FILTER_HELP: &str = " Type to filter \u{00b7} Enter to apply \u{00b7} Esc to cancel";
const MAIN_HELP: &str = " q quit \u{00b7} j/k nav \u{00b7} Enter select \u{00b7} / filter \u{00b7} K kill \u{00b7} F force \u{00b7} r refresh";

pub fn render(w: &mut impl Write, app: &App) -> io::Result<()> {
    let (cols, _) = terminal::size()?;
    let cols = cols as usize;

    if cols < 40 {
        return Ok(());
    }

    let proc_w = cols.saturating_sub(FIXED_W).max(8);

    queue!(w, cursor::MoveTo(0, app.start_row))?;

    render_header(w, cols, app)?;
    render_col_headers(w, cols, proc_w)?;
    render_rows(w, cols, proc_w, app)?;
    render_footer(w, cols, app)?;

    let sel_y = app.start_row as usize + 2 + app.selected - app.scroll_offset;

    if let Some((pid, ref name)) = app.confirm_kill {
        render_confirm_popup(w, cols, sel_y, app.confirm_force, pid, name)?;
    } else if let Some(menu) = &app.action_menu {
        render_action_popup(w, cols, sel_y, menu)?;
    }

    w.flush()
}

fn render_header(w: &mut impl Write, cols: usize, app: &App) -> io::Result<()> {
    let title = if app.filter_mode {
        format!(" portit \u{2014} filter: {}\u{258c}", app.filter)
    } else if !app.filter.is_empty() {
        format!(" portit \u{2014} filter: [{}]", app.filter)
    } else {
        format!(" portit \u{2014} {} ports", app.filtered_entries.len())
    };

    queue!(
        w,
        Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::White),
        SetBackgroundColor(Color::Blue),
        Print(pad_line(&title, cols)),
        ResetColor,
        cursor::MoveToNextLine(1),
    )
}

fn render_col_headers(w: &mut impl Write, cols: usize, proc_w: usize) -> io::Result<()> {
    let line = format_row("PID", "Process", "Proto", "Address", "Port", proc_w);

    queue!(
        w,
        Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::Yellow),
        SetAttribute(Attribute::Bold),
        Print(pad_line(&line, cols)),
        SetAttribute(Attribute::Reset),
        ResetColor,
        cursor::MoveToNextLine(1),
    )
}

fn render_rows(w: &mut impl Write, cols: usize, proc_w: usize, app: &App) -> io::Result<()> {
    let visible = app.visible_rows;
    let end = (app.scroll_offset + visible).min(app.filtered_entries.len());

    for i in app.scroll_offset..end {
        let idx = app.filtered_entries[i];
        let e = &app.entries[idx];
        let line = format_row(
            &e.pid.to_string(),
            &e.process_name,
            "TCP",
            &e.address,
            &e.port.to_string(),
            proc_w,
        );

        render_row_line(w, cols, &line, i == app.selected)?;
    }

    // clear leftover rows if entries < visible
    for _ in end.saturating_sub(app.scroll_offset)..visible {
        queue!(w, Clear(ClearType::CurrentLine), cursor::MoveToNextLine(1))?;
    }

    Ok(())
}

fn render_footer(w: &mut impl Write, cols: usize, app: &App) -> io::Result<()> {
    queue!(w, Clear(ClearType::CurrentLine))?;

    let text = if app.filter_mode {
        FILTER_HELP
    } else {
        MAIN_HELP
    };
    render_status_line(w, cols, text)
}

fn format_row(pid: &str, proc: &str, proto: &str, addr: &str, port: &str, proc_w: usize) -> String {
    format!(
        " {:<PID_W$}{:<pw$}{:<PROTO_W$}{:<ADDR_W$}{:<PORT_W$}",
        truncate(pid, PID_W),
        truncate(proc, proc_w),
        truncate(proto, PROTO_W),
        truncate(addr, ADDR_W),
        truncate(port, PORT_W),
        pw = proc_w,
    )
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}

fn pad_line(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{s:<width$}")
    }
}

fn render_row_line(w: &mut impl Write, cols: usize, line: &str, selected: bool) -> io::Result<()> {
    queue!(w, Clear(ClearType::CurrentLine))?;
    if selected {
        queue!(
            w,
            SetAttribute(Attribute::Reverse),
            Print(pad_line(line, cols)),
            SetAttribute(Attribute::Reset),
            ResetColor,
        )?;
    } else {
        queue!(w, Print(pad_line(line, cols)))?;
    }
    queue!(w, cursor::MoveToNextLine(1))
}

fn render_status_line(w: &mut impl Write, cols: usize, text: &str) -> io::Result<()> {
    queue!(
        w,
        SetForegroundColor(Color::DarkGrey),
        SetBackgroundColor(Color::Black),
        Print(pad_line(text, cols)),
        ResetColor,
    )
}

struct PopupLayout {
    x: u16,
    y: u16,
    h_bar: String,
}

fn popup_layout(cols: usize, sel_y: usize, inner_w: usize) -> PopupLayout {
    PopupLayout {
        x: (cols.saturating_sub(inner_w + 2) / 2) as u16,
        y: (sel_y + 1) as u16,
        h_bar: "\u{2500}".repeat(inner_w),
    }
}

fn popup_top(h_bar: &str) -> String {
    format!("\u{250c}{h_bar}\u{2510}")
}

fn popup_bottom(h_bar: &str) -> String {
    format!("\u{2514}{h_bar}\u{2518}")
}

fn render_confirm_popup(
    w: &mut impl Write,
    cols: usize,
    sel_y: usize,
    force: bool,
    pid: u32,
    name: &str,
) -> io::Result<()> {
    let sig = if force { "SIGKILL" } else { "SIGTERM" };
    let msg = format!(" Kill {} (PID {}) with {}? [y/n] ", name, pid, sig);
    let layout = popup_layout(cols, sel_y, msg.len());

    queue!(
        w,
        cursor::MoveTo(layout.x, layout.y),
        SetForegroundColor(Color::White),
        SetBackgroundColor(Color::Red),
        Print(popup_top(&layout.h_bar)),
        cursor::MoveTo(layout.x, layout.y + 1),
        Print(format!("\u{2502}{msg}\u{2502}")),
        cursor::MoveTo(layout.x, layout.y + 2),
        Print(popup_bottom(&layout.h_bar)),
        ResetColor,
        SetAttribute(Attribute::Reset),
    )
}

fn render_action_popup(
    w: &mut impl Write,
    cols: usize,
    sel_y: usize,
    menu: &crate::app::ActionMenu,
) -> io::Result<()> {
    let inner_w = ACTIONS.iter().map(|a| a.len() + 4).max().unwrap_or(16);
    let layout = popup_layout(cols, sel_y, inner_w);

    queue!(
        w,
        cursor::MoveTo(layout.x, layout.y),
        SetForegroundColor(Color::Cyan),
        SetBackgroundColor(Color::Black),
        Print(popup_top(&layout.h_bar)),
    )?;

    for (i, action) in ACTIONS.iter().enumerate() {
        let marker = if i == menu.selected {
            "\u{25b8} "
        } else {
            "  "
        };
        queue!(w, cursor::MoveTo(layout.x, layout.y + 1 + i as u16))?;
        if i == menu.selected {
            queue!(
                w,
                Print("\u{2502}"),
                SetForegroundColor(Color::Black),
                SetBackgroundColor(Color::Cyan),
                Print(format!(" {marker}{:<w$}", action, w = inner_w - 4)),
                SetForegroundColor(Color::Cyan),
                SetBackgroundColor(Color::Black),
                Print(" \u{2502}"),
            )?;
        } else {
            queue!(
                w,
                Print(format!(
                    "\u{2502} {marker}{:<w$} \u{2502}",
                    action,
                    w = inner_w - 4
                )),
            )?;
        }
    }

    queue!(
        w,
        cursor::MoveTo(layout.x, layout.y + 1 + ACTIONS.len() as u16),
        Print(popup_bottom(&layout.h_bar)),
        ResetColor,
        SetAttribute(Attribute::Reset),
    )
}
