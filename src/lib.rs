//! Main application logic and control flow.
//!
//! This module orchestrates the entire application by:
//! 1. Initializing the [`Node`] tree via filesystem scanning.
//! 2. Managing the main event loop.
//! 3. Bridging the data model with the UI rendering and state updates.

use crate::model::Node;
use crate::ui_state::UiState;
use std::path::PathBuf;

pub mod model;
mod ui;
mod ui_state;

/// The core application container.
///
/// It owns the root [`Node`] tree, ensuring the data stays alive
/// for the duration of the program.
pub struct App {
    pub node: Node,
}

impl App {
    /// Initializes the application by scanning the provided directory path.
    ///
    /// # Errors
    /// Returns an error if the path is invalid or inaccessible.
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let root = Node::scan(path)?;
        Ok(Self { node: root })
    }
    /// Creates a fresh [`UiState`] tied to the lifetime of the `App`'s node tree.
    fn create_ui_state(&self) -> UiState<'_> {
        UiState::new(&self.node)
    }

    /// Starts the main application loop.
    ///
    /// This method handles the "Render-Input-Update" cycle:
    /// 1. Draws the current state to the terminal.
    /// 2. Waits for and parses user input.
    /// 3. Updates the UI state or exits based on user actions.
    ///
    /// # Errors
    /// Returns an error if a fatal I/O or parsing issue occurs.
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
                Err(e) => eprintln!("⚠️ {e}"),
            }
        }
    }
}
