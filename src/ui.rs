//! UI渲染与输入处理
//!
//! 本模块将内部的 [`UiState`] 转换为人类可读的终端界面，并将原始用户按键转换为可操作的 [`Action`]。

use crate::model::NodeKind::*;
use crate::theme::Color;
use crate::ui_state::{Action, SortMode, UiState, ViewItem};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use std::io::{self, Write};

/// raw mode守卫，析构时恢复
#[allow(dead_code)]
pub struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

/// 渲染文件树
pub fn render(state: &UiState) {
    let view = state.flatten_view();
    let total = view.len();
    let max_idx_width = total.saturating_sub(1).to_string().len().max(1);
    let height = state.viewport_height.max(1);

    // 计算视口范围
    let cursor = state.cursor.min(total.saturating_sub(1));
    let start = if total <= height {
        0
    } else if cursor + 1 <= height {
        0
    } else {
        (cursor + 1 - height).min(total.saturating_sub(height))
    };
    let end = (start + height).min(total);
    let remaining_above = start;
    let remaining_below = total.saturating_sub(end);

    // 计算大小范围（用于渐变色）
    let (dir_min, dir_max) = size_range(&view, true).unwrap_or((0, 0));
    let (file_min, file_max) = size_range(&view, false).unwrap_or((0, 0));

    // 清屏
    print!("\x1b[2J\x1b[H");

    // 标题
    println!(
        "--- File Tree (Total: {}, Showing: {}-{}) ---",
        total,
        start,
        end.saturating_sub(1)
    );
    if remaining_above > 0 || remaining_below > 0 {
        println!(
            "(More: above {}, below {})",
            remaining_above, remaining_below
        );
    }

    // 渲染每一行
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
        let size_str = format_size(size);

        let is_selected = index == cursor;
        let (hl_start, hl_end) = if is_selected {
            (
                state.theme.highlight_start.to_ansi().unwrap_or_default(),
                state.theme.highlight_end.to_ansi().unwrap_or_default(),
            )
        } else {
            (String::new(), String::new())
        };
        let selection = if is_selected { ">" } else { " " };

        // 渐变色
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

    // 状态栏
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

    // 帮助栏
    let sort_label = match state.sort_mode {
        SortMode::NameAsc => "name",
        SortMode::SizeDesc => "size",
    };
    print!(
        "[j/k] Move | [Enter/t] Toggle | [s] Sort({}) | [q] Quit | Index: {} > ",
        sort_label, state.input_buffer
    );
    io::stdout().flush().ok();
}

/// 格式化文件大小
fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
    }
}

/// 计算大小范围
fn size_range(view: &[ViewItem<'_>], want_dir: bool) -> Option<(u64, u64)> {
    let mut min: Option<u64> = None;
    let mut max: Option<u64> = None;

    for item in view {
        let is_dir = matches!(item.node.kind(), Directory(_));
        if is_dir != want_dir {
            continue;
        }
        let size = item.node.size();
        min = Some(min.map_or(size, |m| m.min(size)));
        max = Some(max.map_or(size, |m| m.max(size)));
    }

    min.zip(max)
}

/// 计算渐变色
fn gradient_color(
    size: u64,
    min: u64,
    max: u64,
    start: &Color,
    end: &Color,
    fallback: &Color,
) -> String {
    let start_rgb = start.to_rgb().ok();
    let end_rgb = end.to_rgb().ok();

    if let (Some(s), Some(e)) = (start_rgb, end_rgb) {
        let t = if max <= min {
            0.0
        } else {
            (size - min) as f64 / (max - min) as f64
        };
        let (r, g, b) = lerp_rgb(s, e, t);
        return format!("\x1b[38;2;{};{};{}m", r, g, b);
    }

    fallback.to_ansi().unwrap_or_default()
}

/// RGB线性插值
fn lerp_rgb(s: (u8, u8, u8), e: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| ((a as f64) + (b as f64 - a as f64) * t).round() as u8;
    (lerp(s.0, e.0), lerp(s.1, e.1), lerp(s.2, e.2))
}

/// 读取用户输入
pub fn get_input() -> anyhow::Result<Action> {
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            use KeyCode::*;
            match key.code {
                Up | Char('k' | 'K') => return Ok(Action::MoveUp),
                Down | Char('j' | 'J') => return Ok(Action::MoveDown),
                Enter => return Ok(Action::Enter),
                Backspace => return Ok(Action::InputBackspace),
                Char('q' | 'Q') => return Ok(Action::Quit),
                Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(Action::Quit);
                }
                Char('t' | 'T') => return Ok(Action::ToggleAtCursor),
                Char('s' | 'S') => return Ok(Action::ToggleSort),
                Char(ch) if ch.is_ascii_digit() => return Ok(Action::InputDigit(ch)),
                _ => {}
            }
        }
    }
}
