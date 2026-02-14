//! Logic for managing the UI state of the file explorer.
//!
//! This module provides [`UiState`], which tracks which nodes are expanded
//! and projects the hierarchical tree structure into a linear list for rendering.

use crate::model::{Node, NodeKind::*};
use crate::theme::Theme;
use anyhow::bail;

/// Actions for [`UiState::update`].
pub enum Action {
    Toggle(usize),
    ToggleAtCursor,
    MoveUp,
    MoveDown,
    Enter,
    InputDigit(char),
    InputBackspace,
    Quit,
}

#[derive(Clone, Debug)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}

pub struct ViewItem<'a> {
    pub node: &'a Node,
    pub depth: usize,
}

/// Represents the visual state of the tree at a given moment.
///
/// It maintains references to the original [`Node`] tree and keeps track of
/// which directory nodes are currently expanded in the view.
pub struct UiState<'a> {
    pub root: &'a Node,
    /// List of nodes currently shown as "expanded".
    pub expanded_nodes: Vec<&'a Node>,
    /// Current selected row index in the flattened view.
    pub cursor: usize,
    /// Max rows to render in one screenful.
    pub viewport_height: usize,
    /// Pending numeric input for index toggle.
    pub input_buffer: String,
    /// Status line message for hints/errors.
    pub status: Option<StatusMessage>,
    /// Theme for UI colors.
    pub theme: Theme,
}

impl<'a> UiState<'a> {
    /// Creates a new UI state with the root node expanded by default.
    pub fn new(root: &'a Node, theme: Theme) -> Self {
        Self {
            root,
            expanded_nodes: vec![root],
            cursor: 0,
            viewport_height: 20,
            input_buffer: String::new(),
            status: None,
            theme,
        }
    }

    /// Projects the tree structure into a flat list of visible items.
    ///
    /// This takes the expansion state into account, only including children of
    /// nodes present in `expanded_nodes`.
    pub fn flatten_view(&self) -> Vec<ViewItem<'a>> {
        let mut items = Vec::new();
        self.collect_recursive(self.root, 0, &mut items);
        items
    }

    /// Recursively collects visible nodes into a flat vector.
    fn collect_recursive(&self, node: &'a Node, depth: usize, items: &mut Vec<ViewItem<'a>>) {
        items.push(ViewItem { node, depth });

        if let Directory(prop) = node.kind()
            && self.expanded_nodes.contains(&node)
        {
            for child in prop.children() {
                self.collect_recursive(child, depth + 1, items);
            }
        }
    }

    /// Moves the cursor by a delta and clamps it within the view length.
    fn move_cursor(&mut self, delta: isize, view_len: usize) {
        if view_len == 0 {
            self.cursor = 0;
            return;
        }

        let new_cursor = if delta < 0 {
            self.cursor.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            let step = delta as usize;
            (self.cursor + step).min(view_len - 1)
        };

        self.cursor = new_cursor;
    }

    /// Toggles the directory at the current cursor position.
    fn toggle_at_cursor(&mut self) -> anyhow::Result<()> {
        self.toggle_by_index(self.cursor)
    }

    fn set_error(&mut self, message: impl Into<String>) {
        self.status = Some(StatusMessage {
            text: message.into(),
            is_error: true,
        });
    }

    fn clear_status(&mut self) {
        self.status = None;
    }

    /// Logic for toggling a directory node by its row index in the current view.
    fn toggle_by_index(&mut self, index: usize) -> anyhow::Result<()> {
        let view = self.flatten_view();

        let item = view
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("Index {index} not found!"))?;
        let target_node = item.node;

        if let File = target_node.kind() {
            bail!("{target_node:?} cannot be toggled because it is a file.");
        }

        let pos = self.expanded_nodes.iter().position(|&x| x == target_node);
        // remove if expanded, push if collapsed.
        match pos {
            Some(idx) => {
                // 已存在 -> 移除（折叠）
                self.expanded_nodes.remove(idx);
            }
            None => {
                // 不存在 -> 添加（展开）
                self.expanded_nodes.push(target_node);
            }
        }

        let view_len = self.flatten_view().len();
        if self.cursor >= view_len {
            self.cursor = view_len.saturating_sub(1);
        }

        Ok(())
    }

    /// Updates the UI state based on the provided [`Action`].
    ///
    /// # Returns
    /// - `Ok(true)`: The state was updated and the application should continue.
    /// - `Ok(false)`: The user requested to quit.
    ///
    /// # Errors
    /// Returns an error if the action (e.g., toggling an index) is invalid.
    pub fn update(&mut self, action: Action) -> anyhow::Result<bool> {
        let view_len = self.flatten_view().len();

        match action {
            Action::MoveUp => {
                self.input_buffer.clear();
                self.clear_status();
                self.move_cursor(-1, view_len);
                Ok(true)
            }
            Action::MoveDown => {
                self.input_buffer.clear();
                self.clear_status();
                self.move_cursor(1, view_len);
                Ok(true)
            }
            Action::ToggleAtCursor => {
                self.input_buffer.clear();
                match self.toggle_at_cursor() {
                    Ok(()) => self.clear_status(),
                    Err(e) => self.set_error(e.to_string()),
                }
                Ok(true)
            }
            Action::Toggle(index) => {
                self.input_buffer.clear();
                match self.toggle_by_index(index) {
                    Ok(()) => {
                        self.cursor = index.min(self.flatten_view().len().saturating_sub(1));
                        self.clear_status();
                    }
                    Err(e) => self.set_error(e.to_string()),
                }
                Ok(true)
            }
            Action::Enter => {
                if self.input_buffer.is_empty() {
                    match self.toggle_at_cursor() {
                        Ok(()) => self.clear_status(),
                        Err(e) => self.set_error(e.to_string()),
                    }
                } else {
                    let index = match self.input_buffer.parse::<usize>() {
                        Ok(index) => index,
                        Err(_) => {
                            self.set_error(format!("Invalid index: {}", self.input_buffer));
                            self.input_buffer.clear();
                            return Ok(true);
                        }
                    };
                    match self.toggle_by_index(index) {
                        Ok(()) => {
                            self.cursor = index.min(self.flatten_view().len().saturating_sub(1));
                            self.clear_status();
                        }
                        Err(e) => self.set_error(e.to_string()),
                    }
                }
                self.input_buffer.clear();
                Ok(true)
            }
            Action::InputDigit(ch) => {
                if ch.is_ascii_digit() {
                    self.input_buffer.push(ch);
                }
                self.clear_status();
                Ok(true)
            }
            Action::InputBackspace => {
                self.input_buffer.pop();
                self.clear_status();
                Ok(true)
            }
            Action::Quit => Ok(false),
        }
    }
}
