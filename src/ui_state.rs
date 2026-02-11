//! Logic for managing the UI state of the file explorer.
//!
//! This module provides [`UiState`], which tracks which nodes are expanded
//! and projects the hierarchical tree structure into a linear list for rendering.

use crate::model::{Node, NodeKind::*};
use anyhow::bail;

/// Actions for [`UiState::update`].
pub enum Action {
    Toggle(usize),
    Quit,
}

/// Items to be displayed.
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
}

impl<'a> UiState<'a> {
    /// Creates a new UI state with the root node expanded by default.
    pub fn new(root: &'a Node) -> Self {
        Self {
            root,
            expanded_nodes: vec![root],
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
        match action {
            Action::Toggle(index) => {
                self.toggle_by_index(index)?;
                Ok(true)
            }
            Action::Quit => Ok(false),
        }
    }
}
