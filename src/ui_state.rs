//! UI状态管理
//!
//! 本模块提供了 [`UiState`]，用于跟踪节点的展开状态，并将层次树结构投影到线性列表中以便渲染。

use crate::model::{Node, NodeKind::*};
use crate::theme::Theme;
use anyhow::bail;

/// UI动作
#[allow(dead_code)]
pub enum Action {
    Toggle(usize),      // 按索引切换
    ToggleAtCursor,     // 切换光标处
    MoveUp,             // 上移
    MoveDown,           // 下移
    Enter,              // 确认
    InputDigit(char),   // 输入数字
    InputBackspace,     // 退格
    ToggleSort,         // 切换排序
    Quit,               // 退出
}

/// 排序模式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortMode {
    NameAsc,  // 按名称升序
    SizeDesc, // 按大小降序
}

/// 状态消息
#[derive(Clone, Debug)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}

/// 视图项
pub struct ViewItem<'a> {
    pub node: &'a Node,
    pub depth: usize,
}

/// UI状态
pub struct UiState<'a> {
    pub root: &'a Node,
    pub expanded_nodes: Vec<&'a Node>, // 已展开节点
    pub cursor: usize,                 // 光标位置
    pub viewport_height: usize,        // 视口高度
    pub input_buffer: String,          // 输入缓冲
    pub status: Option<StatusMessage>, // 状态消息
    pub theme: Theme,                  // 主题
    pub sort_mode: SortMode,           // 排序模式
}

impl<'a> UiState<'a> {
    /// 创建新状态，默认展开根节点
    pub fn new(root: &'a Node, theme: Theme) -> Self {
        Self {
            root,
            expanded_nodes: vec![root],
            cursor: 0,
            viewport_height: 20,
            input_buffer: String::new(),
            status: None,
            theme,
            sort_mode: SortMode::SizeDesc,
        }
    }

    /// 展平树为可见项列表
    pub fn flatten_view(&self) -> Vec<ViewItem<'a>> {
        let mut items = Vec::new();
        self.collect_recursive(self.root, 0, &mut items);
        items
    }

    /// 递归收集可见节点
    fn collect_recursive(&self, node: &'a Node, depth: usize, items: &mut Vec<ViewItem<'a>>) {
        items.push(ViewItem { node, depth });

        if let Directory(prop) = node.kind()
            && self.expanded_nodes.contains(&node)
        {
            let mut children: Vec<&Node> = prop.children().iter().collect();
            children.sort_by(|a, b| self.compare_nodes(a, b));
            for child in children {
                self.collect_recursive(child, depth + 1, items);
            }
        }
    }

    /// 比较节点（目录优先，再按排序模式）
    fn compare_nodes(&self, a: &Node, b: &Node) -> std::cmp::Ordering {
        match (a.kind(), b.kind()) {
            (Directory(_), File) => std::cmp::Ordering::Less,
            (File, Directory(_)) => std::cmp::Ordering::Greater,
            _ => match self.sort_mode {
                SortMode::NameAsc => a.path().cmp(b.path()),
                SortMode::SizeDesc => b.size().cmp(&a.size()).then_with(|| a.path().cmp(b.path())),
            },
        }
    }

    /// 移动光标
    fn move_cursor(&mut self, delta: isize, view_len: usize) {
        if view_len == 0 {
            self.cursor = 0;
            return;
        }
        let new_cursor = if delta < 0 {
            self.cursor.saturating_sub(delta.unsigned_abs())
        } else {
            (self.cursor + delta as usize).min(view_len - 1)
        };
        self.cursor = new_cursor;
    }

    /// 切换光标处目录
    fn toggle_at_cursor(&mut self) -> anyhow::Result<()> {
        self.toggle_by_index(self.cursor)
    }

    /// 设置错误消息
    fn set_error(&mut self, message: impl Into<String>) {
        self.status = Some(StatusMessage {
            text: message.into(),
            is_error: true,
        });
    }

    /// 清除状态消息
    fn clear_status(&mut self) {
        self.status = None;
    }

    /// 按索引切换目录展开/折叠
    fn toggle_by_index(&mut self, index: usize) -> anyhow::Result<()> {
        let view = self.flatten_view();
        let item = view
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("Index {index} not found!"))?;
        let target_node = item.node;

        if let File = target_node.kind() {
            bail!("Cannot toggle file");
        }

        // 切换展开状态
        match self.expanded_nodes.iter().position(|&x| x == target_node) {
            Some(idx) => { self.expanded_nodes.remove(idx); }
            None => { self.expanded_nodes.push(target_node); }
        }

        // 调整光标
        let view_len = self.flatten_view().len();
        if self.cursor >= view_len {
            self.cursor = view_len.saturating_sub(1);
        }
        Ok(())
    }

    /// 处理动作，返回是否继续运行
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
                        Ok(i) => i,
                        Err(_) => {
                            self.set_error(format!("Invalid: {}", self.input_buffer));
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
            Action::ToggleSort => {
                self.input_buffer.clear();
                self.clear_status();
                self.sort_mode = match self.sort_mode {
                    SortMode::NameAsc => SortMode::SizeDesc,
                    SortMode::SizeDesc => SortMode::NameAsc,
                };
                Ok(true)
            }
            Action::Quit => Ok(false),
        }
    }
}
