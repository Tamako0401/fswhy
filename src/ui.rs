//! UI rendering and input handling logic.
//!
//! This module translates the internal [`UiState`] into a human-readable
//! terminal interface and converts raw user keystrokes into actionable [`Action`]s.

use crate::model::NodeKind::*;
use crate::ui_state::Action::Toggle;
use crate::ui_state::{Action, UiState};
use anyhow::bail;
use std::io;
use std::io::Write;

/// Renders the current file tree to the standard output.
///
/// This function calculates the necessary padding for indices to ensure
/// vertical alignment and formats file sizes into human-readable units
/// (B, KB, MB).
/// ```no_run
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
    let max_idx_width = view.len().to_string().len();

    println!("\n--- File Tree (Total: {}) ---", view.len());
    for (index, item) in view.iter().enumerate() {
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

        println!(
            "{} {}{} {} ({})",
            idx_str,
            prefix,
            icon,
            item.node
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            size_str
        );
    }
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
    print!("\n[Index] Toggle Dir | [q] Quit > ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    match trimmed {
        "q" | "Q" => Ok(Action::Quit),
        num_str => {
            if let Ok(index) = num_str.parse::<usize>() {
                Ok(Toggle(index))
            } else {
                bail!("Invalid input: '{}'", num_str)
            }
        }
    }
}
