use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Node {
    path: PathBuf,
    size: u64,
    kind: NodeKind,
}

#[derive(Debug)]
enum NodeKind {
    File,
    Directory(DirProperty),
}

#[derive(Debug)]
pub struct DirProperty {
    children: Vec<Node>,
    expanded: bool,
}

impl Node {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.kind, NodeKind::Directory(_))
    }

    pub fn children(&self) -> Option<&[Node]> {
        match &self.kind {
            NodeKind::Directory(dir) => Some(&dir.children),
            _ => None,
        }
    }

    pub fn scan(path: PathBuf) -> anyhow::Result<Node> {
        let meta = std::fs::metadata(&path)?;

        if meta.is_dir() {
            let mut children = Vec::new();
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                children.push(Node::scan(entry.path())?);
            }

            Ok(Node {
                path,
                size: meta.len(),
                kind: NodeKind::Directory(DirProperty {
                    children,
                    expanded: false,
                }),
            })
        } else {
            Ok(Node {
                path,
                size: meta.len(),
                kind: NodeKind::File,
            })
        }
    }

    pub fn expand(&mut self) {
        if let NodeKind::Directory(dir) = &mut self.kind {
            dir.expanded = !dir.expanded;
        }
    }
}
