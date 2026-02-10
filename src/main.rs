use anyhow::Result;
use std::env;
use std::path::PathBuf;

mod model;
mod scan;

use model::Node;

fn main() -> Result<()> {
    let root_path: PathBuf = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);

    let root = Node::scan(root_path)?;

    print_tree(&root, 0);

    Ok(())
}

fn print_tree(node: &Node, indent: usize) {
    let prefix = "  ".repeat(indent);

    println!("{}{} ({})", prefix, node.path().display(), node.size());

    if let Some(children) = node.children() {
        for child in children {
            print_tree(child, indent + 1);
        }
    }
}
