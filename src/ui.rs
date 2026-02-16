//! UI rendering and input handling logic.
//!
//! This module translates the internal [`UiState`] into a human-readable
//! terminal interface and converts raw user keystrokes into actionable [`Action`]s.

use crate::model::NodeKind::*;
use crate::ui_state::{Action, SortMode, UiState};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use std::io;
use std::io::Write;

/// Enables raw mode for immediate key handling and restores it on drop.
pub struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

/// Returns terminal height in rows
/// Renders the current file tree to the standard output.
///
/// This function calculates the necessary padding for indices to ensure
/// vertical alignment and formats file sizes into human-readable units
/// (B, KB, MB).
/// ```text
/// --- File Tree (Total: 9) ---
/// 0 [-] fswhy (45.9 MB)
/// 1   [+] .git (36.3 KB)
/// 2   [+] .idea (8.1 KB)
/// 3   [+] src (7.4 KB)
/// 4   [+] target (45.8 MB)
/// 5       .gitignore (14 B)
/// 6       Cargo.lock (371 B)
/// 7       Cargo.toml (88 B)
/// 8       README.md (607 B)
/// ```
/// # Arguments
/// * `state` - A reference to the [`UiState`] containing the tree data and
///   the set of expanded nodes.
pub fn render(state: &UiState) {
    let view = state.flatten_view();
    let total = view.len();
    let max_idx_width = total.saturating_sub(1).to_string().len().max(1);
    let height = state.viewport_height.max(1);

    let cursor = state.cursor.min(total.saturating_sub(1));
    let start = if total <= height {
        0
    } else if cursor + 1 <= height {
        0
    } else {
        let max_start = total.saturating_sub(height);
        (cursor + 1 - height).min(max_start)
    };
    let end = (start + height).min(total);
    let remaining_above = start;
    let remaining_below = total.saturating_sub(end);

    let (dir_min, dir_max) = size_range(&view, true).unwrap_or((0, 0));
    let (file_min, file_max) = size_range(&view, false).unwrap_or((0, 0));

    // Cls and move cursor to home
    print!("\x1b[2J\x1b[H");

    println!(
        "--- File Tree (Total: {}, Showing: {}-{}) ---",
        total,
        start,
        end.saturating_sub(1)
    );
    if remaining_above > 0 || remaining_below > 0 {
        println!("(More: above {}, below {})", remaining_above, remaining_below);
    }

    for (index, item) in view.iter().enumerate().skip(start).take(end - start) {
        let prefix = "  ".repeat(item.depth);
        let idx_str = format!("{:width$}", index, width = max_idx_width);
        let icon = match item.node.kind() {
            Directory(_) => {
                if state.expanded_nodes.contains(&item.node) {
                    "[-]"
                } else {
                    "[+]"
                }
            }
            File => "   ",
        };

        let size = item.node.size();
        let size_str = if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
        };

        let is_selected = index == cursor;
        let (hl_start, hl_end) = if is_selected {
            (
                state
                    .theme
                    .highlight_start
                    .to_ansi()
                    .unwrap_or_default(),
                state.theme.highlight_end.to_ansi().unwrap_or_default(),
            )
        } else {
            (String::new(), String::new())
        };
        let selection = if is_selected { ">" } else { " " };
        let name_color = match item.node.kind() {
            Directory(_) => gradient_color(
                size,
                dir_min,
                dir_max,
                &state.theme.dir_gradient_start,
                &state.theme.dir_gradient_end,
                &state.theme.dir,
            ),
            File => gradient_color(
                size,
                file_min,
                file_max,
                &state.theme.file_gradient_start,
                &state.theme.file_gradient_end,
                &state.theme.file,
            ),
        };
        let fg_reset = state.theme.fg_reset.to_ansi().unwrap_or_default();

        println!(
            "{}{} {}{} {} {}{}{} ({}){}",
            hl_start,
            selection,
            idx_str,
            prefix,
            icon,
            name_color,
            item.node
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            fg_reset,
            size_str,
            hl_end
        );
    }

    if let Some(status) = &state.status {
        let color = if status.is_error {
            state.theme.error.to_ansi().unwrap_or_default()
        } else {
            String::new()
        };
        let reset = state.theme.reset.to_ansi().unwrap_or_default();
        println!("{}{}{}", color, status.text, reset);
    } else {
        println!();
    }

    let sort_label = match state.sort_mode {
        SortMode::NameAsc => "name",
        SortMode::SizeDesc => "size",
    };
    print!(
        "[j/k] Move | [Enter/t] Toggle | [index] Toggle | [s] Sort({}) | [q] Quit | Index: {} > ",
        sort_label,
        state.input_buffer
    );
    io::stdout().flush().ok();
}

fn size_range(view: &[crate::ui_state::ViewItem<'_>], want_dir: bool) -> Option<(u64, u64)> {
    let mut min = None;
    let mut max = None;

    for item in view {
        let is_dir = matches!(item.node.kind(), Directory(_));
        if is_dir != want_dir {
            continue;
        }
        let size = item.node.size();
        min = Some(min.map_or(size, |m: u64| m.min(size)));
        max = Some(max.map_or(size, |m: u64| m.max(size)));

    }

    match (min, max) {
        (Some(min), Some(max)) => Some((min, max)),
        _ => None,
    }
}

fn gradient_color(
    size: u64,
    min: u64,
    max: u64,
    start: &crate::theme::Color,
    end: &crate::theme::Color,
    fallback: &crate::theme::Color,
) -> String {
    let start_rgb = start.to_rgb().ok();
    let end_rgb = end.to_rgb().ok();
    if let (Some(start_rgb), Some(end_rgb)) = (start_rgb, end_rgb) {
        let t = if max <= min {
            0.0
        } else {
            (size.saturating_sub(min)) as f64 / (max - min) as f64
        };
        let (r, g, b) = lerp_rgb(start_rgb, end_rgb, t);
        return format!("\x1b[38;2;{};{};{}m", r, g, b);
    }

    fallback.to_ansi().unwrap_or_default()
}

fn lerp_rgb(start: (u8, u8, u8), end: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| -> u8 { ((a as f64) + (b as f64 - a as f64) * t).round() as u8 };
    (lerp(start.0, end.0), lerp(start.1, end.1), lerp(start.2, end.2))
}

/// Prompts the user for input and parses it into an [`Action`].
///
/// # Errors
/// Returns an error if the input is neither a quit command ('q') nor
/// a valid numeric index.
///
/// # Panics
/// This function will not panic under normal circumstances, but it may
/// return an error if `stdin` is unavailable.
pub fn get_input() -> anyhow::Result<Action> {
    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                use KeyCode::*;

                match key.code {
                    Up => return Ok(Action::MoveUp),
                    Down => return Ok(Action::MoveDown),
                    Enter => return Ok(Action::Enter),
                    Backspace => return Ok(Action::InputBackspace),
                    Char('q' | 'Q') => return Ok(Action::Quit),
                    Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(Action::Quit);
                    }
                    Char('j' | 'J') => return Ok(Action::MoveDown),
                    Char('k' | 'K') => return Ok(Action::MoveUp),
                    Char('t' | 'T') => return Ok(Action::ToggleAtCursor),
                    Char('s' | 'S') => return Ok(Action::ToggleSort),
                    Char(ch) if ch.is_ascii_digit() => return Ok(Action::InputDigit(ch)),
                    _ => {}
                }
            }
            Event::Resize(_, _) => {
                // Ignore
            }
            _ => {}
        }
    }
}
