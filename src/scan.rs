use crate::model::Node;
use anyhow::Result;
use std::path::PathBuf;

pub fn scan(path: PathBuf) -> Result<Node> {
    Node::scan(path)
}
