//! 文件系统数据模型
//!
//! 本模块提供了 [`Node`] 结构体，用于递归表示文件和目录信息，并提供 [`Node::scan`] 方法从实际文件系统构建树形结构。

use crate::model::NodeKind::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

#[derive(PartialOrd, PartialEq, Debug)]
pub struct Node {
    path: PathBuf,
    size: u64,
    kind: NodeKind,
}

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

    /// 递归扫描文件系统，构建节点树
    ///
    /// 此方法构建 [`Node`] 树。通过对子节点的大小求和来计算目录的总大小，并根据特定优先级对条目进行排序：
    /// 1. 目录优先于文件。
    /// 2. 同类型条目按路径字母顺序排序。
    /// 扫描过程中会将进度信息显示到标准错误输出：
    /// - 每扫描 100 项显示一次进度
    /// - 顶层目录会显示详细统计信息
    ///
    /// # 错误
    /// 如果路径不存在或权限不足以读取目录，则返回错误。
    pub fn scan(path: PathBuf) -> anyhow::Result<Node> {
        // 全局计数器，跨所有层级统计
        static TOTAL_COUNT: AtomicUsize = AtomicUsize::new(0);
        TOTAL_COUNT.store(0, Ordering::Relaxed);

        eprintln!("Scanning {}...", path.display());
        let result = Self::scan_with_progress(path, 0, &TOTAL_COUNT);
        eprintln!();
        result
    }

    /// 带进度显示的递归扫描
    ///
    /// 此方法由 [`scan`](Self::scan) 调用，递归构建目录树，同时更新全局原子计数器以显示进度。
    ///
    /// # 参数
    /// * `path` - 要扫描的文件系统路径
    /// * `depth` - 当前递归深度（根目录为 0）
    /// * `total_count` - 用于跟踪扫描总项数的共享原子计数器
    ///
    /// # 进度显示
    /// - 每扫描 100 项向标准错误输出显示一次进度
    /// - 对于深度为 0 或 1 的目录，显示详细统计信息（目录/文件计数、大小、时间），以避免输出过多信息
    ///
    /// # 错误处理
    /// - 跳过无法访问的条目，继续扫描
    /// - 仅对顶层条目（深度 ≤ 1）记录错误到标准错误输出
    fn scan_with_progress(
        path: PathBuf,
        depth: usize,
        total_count: &AtomicUsize,
    ) -> anyhow::Result<Node> {
        let start = Instant::now();
        let meta = std::fs::metadata(&path)?;

        if meta.is_dir() {
            let mut children = Vec::new();
            let mut file_count = 0;
            let mut dir_count = 0;

            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                let child_path = entry.path();

                match Self::scan_with_progress(child_path, depth + 1, total_count) {
                    Ok(child) => {
                        match child.kind() {
                            File => file_count += 1,
                            Directory(_) => dir_count += 1,
                        }
                        children.push(child);

                        let count = total_count.fetch_add(1, Ordering::Relaxed) + 1;

                        // 每100项显示进度
                        if count % 100 == 0 {
                            eprint!("\rScanned {} items...", count);
                            std::io::Write::flush(&mut std::io::stderr()).ok();
                        }
                    }
                    Err(e) => {
                        // 仅顶层目录打印错误
                        if depth <= 1 {
                            eprintln!("\n✗ Skipped: {}", e);
                        }
                    }
                }
            }

            // 目录优先，按路径排序
            children.sort_by(|a, b| match (&a.kind, &b.kind) {
                (Directory(_), File) => std::cmp::Ordering::Less,
                (File, Directory(_)) => std::cmp::Ordering::Greater,
                _ => a.path.cmp(&b.path),
            });

            let total_size: u64 = children.iter().map(|c| c.size).sum();

            // 顶层目录打印统计
            if depth <= 1 {
                eprintln!(
                    "\n✓ {} ({} dirs, {} files, {:.1} MB) in {:.2}s",
                    path.display(),
                    dir_count,
                    file_count,
                    total_size as f64 / 1024.0 / 1024.0,
                    start.elapsed().as_secs_f64(),
                );
            }

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
