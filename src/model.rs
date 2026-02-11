//! Data structures representing the file system hierarchy.
//!
//! This module provides the [`Node`] struct, which recursively captures
//! file and directory information, and a [`Node::scan`] method to build
//! the tree from the actual file system.

use crate::model::NodeKind::*;
use std::path::{Path, PathBuf};

/// A single entity in the file system tree (either a file or a directory).
///
/// Nodes store essential metadata such as path and size. For directories,
/// the size is the cumulative sum of all descendant nodes.
#[derive(PartialOrd, PartialEq, Debug)]
pub struct Node {
    path: PathBuf,
    size: u64,
    kind: NodeKind,
}

/// Specialized data specific to the type of the [`Node`].
#[derive(PartialOrd, PartialEq, Debug)]
pub enum NodeKind {
    File,
    Directory(DirProperty),
}

#[derive(PartialOrd, PartialEq, Debug)]
pub struct DirProperty {
    children: Vec<Node>,
}

impl DirProperty {
    pub fn children(&self) -> &[Node] {
        &self.children
    }
}

impl Node {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Recursively scans the filesystem starting from the given path.
    ///
    /// This method builds a tree of [`Node`]s. It calculates the total size
    /// of directories by summing up their children and sorts entries
    /// based on a specific priority:
    /// 1. Directories come before files.
    /// 2. Entries of the same type are sorted alphabetically by path.
    ///
    /// # Errors
    /// Returns an error if the path does not exist or if permissions are
    /// insufficient to read the directory.
    pub fn scan(path: PathBuf) -> anyhow::Result<Node> {
        let meta = std::fs::metadata(&path)?;

        if meta.is_dir() {
            let mut children = Vec::new();
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                children.push(Node::scan(entry.path())?);
            }

            children.sort_by(|a, b| {
                match (&a.kind, &b.kind) {
                    (Directory(_), File) => std::cmp::Ordering::Less, // a是目录，b是文件 -> a排前
                    (File, Directory(_)) => std::cmp::Ordering::Greater, // a是文件，b是目录 -> b排前
                    _ => a.path.cmp(&b.path),                            // 同类按路径名排序
                }
            });

            let total_size: u64 = children.iter().map(|c| c.size).sum();
            Ok(Node {
                path,
                size: total_size,
                kind: Directory(DirProperty { children }),
            })
        } else {
            Ok(Node {
                path,
                size: meta.len(),
                kind: File,
            })
        }
    }
}
