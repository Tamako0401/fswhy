//! 应用主入口与事件循环

use crate::model::Node;
use crate::theme::load_theme_from_env_or_default;
use crate::ui_state::UiState;
use std::path::PathBuf;

pub mod model;
mod theme;
mod ui;
mod ui_state;

/// 应用容器，持有文件树根节点
pub struct App {
    pub node: Node,
}

impl App {
    /// 扫描指定路径并初始化应用
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let root = Node::scan(path)?;
        Ok(Self { node: root })
    }

    /// 创建UI状态
    fn create_ui_state(&self) -> UiState<'_> {
        let theme = load_theme_from_env_or_default();
        UiState::new(&self.node, theme)
    }

    /// 主循环：渲染 → 输入 → 更新
    pub fn run(&self) -> anyhow::Result<()> {
        let mut state = self.create_ui_state();
        loop {
            ui::render(&state);

            let action = match ui::get_input() {
                Ok(action) => action,
                Err(e) => {
                    eprintln!("⚠️ Input error: {e}");
                    continue;
                }
            };

            match state.update(action) {
                Ok(false) => break Ok(()),
                Ok(true) => continue,
                Err(e) => eprintln!("⚠️{e}"),
            }
        }
    }
}
