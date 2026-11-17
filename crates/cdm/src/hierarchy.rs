//! # Hierarchy and Structural Tree Models
//!
//! Represents nested document outlines (headings/sections) or repository file trees
//! in the Canonical Context Document Model (`CdmDocument`).

use serde::{Deserialize, Serialize};

/// Type of node within a hierarchical structure tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HierarchyKind {
    /// Root node of the entire repository or document tree.
    Root,
    /// Directory container holding child files and subdirectories.
    Directory,
    /// Source code or document file node.
    File,
    /// Logical section or sub-component.
    Section,
    /// Heading level container in an outline.
    Heading,
}

impl HierarchyKind {
    /// Returns a human-readable identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Directory => "directory",
            Self::File => "file",
            Self::Section => "section",
            Self::Heading => "heading",
        }
    }
}

/// A node in the hierarchical outline or file tree of a CDM document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchyNode {
    /// Unique identifier for this tree node.
    pub id: String,
    /// Display name of the item (e.g. filename `lib.rs` or directory name `crates`).
    pub name: String,
    /// Normalized full relative path from root (e.g. `crates/cdm/src/lib.rs`).
    pub path: String,
    /// Structural kind of this node.
    pub kind: HierarchyKind,
    /// Optional section ID if this node directly corresponds to a `CdmSection`.
    pub section_id: Option<String>,
    /// Nested child nodes.
    pub children: Vec<HierarchyNode>,
    /// Optional byte size of the underlying resource.
    pub byte_size: Option<u64>,
    /// Optional line count spanned.
    pub line_count: Option<u32>,
}

impl HierarchyNode {
    /// Constructs a root hierarchy container.
    #[must_use]
    pub fn root(name: impl Into<String>) -> Self {
        Self {
            id: "root".to_string(),
            name: name.into(),
            path: String::new(),
            kind: HierarchyKind::Root,
            section_id: None,
            children: Vec::new(),
            byte_size: None,
            line_count: None,
        }
    }

    /// Constructs a directory node.
    #[must_use]
    pub fn directory(
        id: impl Into<String>,
        name: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            kind: HierarchyKind::Directory,
            section_id: None,
            children: Vec::new(),
            byte_size: None,
            line_count: None,
        }
    }

    /// Constructs a file node referencing a CDM section.
    #[must_use]
    pub fn file(
        id: impl Into<String>,
        name: impl Into<String>,
        path: impl Into<String>,
        section_id: Option<String>,
        byte_size: Option<u64>,
        line_count: Option<u32>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            kind: HierarchyKind::File,
            section_id,
            children: Vec::new(),
            byte_size,
            line_count,
        }
    }

    /// Adds a child node under this hierarchy node.
    pub fn add_child(&mut self, child: HierarchyNode) {
        self.children.push(child);
    }

    /// Recursively searches for a node with the specified ID.
    #[must_use]
    pub fn find_node(&self, target_id: &str) -> Option<&HierarchyNode> {
        if self.id == target_id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_node(target_id) {
                return Some(found);
            }
        }
        None
    }

    /// Recursively searches for a node with the specified path.
    #[must_use]
    pub fn find_node_by_path(&self, target_path: &str) -> Option<&HierarchyNode> {
        if self.path == target_path {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_node_by_path(target_path) {
                return Some(found);
            }
        }
        None
    }

    /// Returns a pre-order flattened vector of references to all nodes in the tree.
    #[must_use]
    pub fn flatten(&self) -> Vec<&HierarchyNode> {
        let mut list = Vec::new();
        self.collect_nodes(&mut list);
        list
    }

    fn collect_nodes<'a>(&'a self, list: &mut Vec<&'a HierarchyNode>) {
        list.push(self);
        for child in &self.children {
            child.collect_nodes(list);
        }
    }

    /// Computes the total number of nodes in this subtree.
    #[must_use]
    pub fn total_nodes(&self) -> usize {
        1 + self.children.iter().map(Self::total_nodes).sum::<usize>()
    }

    /// Computes the total number of file nodes in this subtree.
    #[must_use]
    pub fn total_files(&self) -> usize {
        let self_file = usize::from(self.kind == HierarchyKind::File);
        self_file + self.children.iter().map(Self::total_files).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_tree_operations() {
        let mut root = HierarchyNode::root("my-repo");
        let mut src_dir = HierarchyNode::directory("dir-src", "src", "src");
        let main_file = HierarchyNode::file(
            "file-main",
            "main.rs",
            "src/main.rs",
            Some("sec-main".to_string()),
            Some(1024),
            Some(50),
        );
        let lib_file = HierarchyNode::file(
            "file-lib",
            "lib.rs",
            "src/lib.rs",
            Some("sec-lib".to_string()),
            Some(2048),
            Some(120),
        );

        src_dir.add_child(main_file);
        src_dir.add_child(lib_file);
        root.add_child(src_dir);

        assert_eq!(root.total_nodes(), 4); // root, src, main.rs, lib.rs
        assert_eq!(root.total_files(), 2);

        let found = root.find_node("file-main");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "main.rs");

        let found_path = root.find_node_by_path("src/lib.rs");
        assert!(found_path.is_some());
        assert_eq!(found_path.unwrap().section_id, Some("sec-lib".to_string()));

        let flat = root.flatten();
        assert_eq!(flat.len(), 4);
    }
}
