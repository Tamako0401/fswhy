//! UI rendering and input handling logic.
//!
//! This module translates the internal [`UiState`] into a human-readable
//! terminal interface and converts raw user keystrokes into actionable [`Action`]s.

use crate::model::NodeKind::*;
use crate::ui_state::{Action, UiState};

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
            Directory(_) => state.theme.dir.to_ansi().unwrap_or_default(),
            File => state.theme.file.to_ansi().unwrap_or_default(),
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

    print!(
        "[j/k] Move | [Enter/t] Toggle | [index] Toggle | [q] Quit | Index: {} > ",
        state.input_buffer
    );
    io::stdout().flush().ok();
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
