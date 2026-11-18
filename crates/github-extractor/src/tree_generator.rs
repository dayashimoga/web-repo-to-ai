//! # Repository Directory Tree Generation and Visualization
//!
//! Generates hierarchical tree structures, visual ASCII / Unicode directory diagrams,
//! and Canonical Context Document Model ([`CdmSection`]) manifest sections ([`SectionKind::Manifest`])
//! representing the complete repository structure for LLM context ingestion.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmSection, CdmSectionBuilder, SectionKind, SectionTokenMetrics};

use crate::classification::ClassifiedFile;
use crate::source::compute_blake3_hash;
use crate::tree::{GitFileMode, GitTreeEntry, GitTreeResponse};
use crate::FileCategory;

/// Formats a byte quantity into a human-readable string (e.g. `512 B`, `1.4 KB`, `2.3 MB`).
#[must_use]
pub fn format_byte_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        let kb = bytes as f64 / KB as f64;
        format!("{kb:.1} KB")
    } else if bytes < GB {
        let mb = bytes as f64 / MB as f64;
        format!("{mb:.1} MB")
    } else {
        let gb = bytes as f64 / GB as f64;
        format!("{gb:.2} GB")
    }
}

/// Returns a concise badge for file category annotations in tree diagrams.
#[must_use]
pub const fn category_badge(cat: FileCategory) -> &'static str {
    match cat {
        FileCategory::SourceCode => "source",
        FileCategory::Documentation => "doc",
        FileCategory::Configuration => "config",
        FileCategory::DependencyManifest => "manifest",
        FileCategory::TestFixture => "test",
        FileCategory::GeneratedOrLockfile => "lockfile",
        FileCategory::Binary => "binary",
        FileCategory::Oversized => "oversized",
        FileCategory::Other => "other",
    }
}

/// Visual indentation styling for rendering directory hierarchies.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreeStyle {
    /// Unicode box-drawing characters (`├── `, `└── `, `│   `).
    #[default]
    Unicode,
    /// Plain ASCII characters (`|-- `, `\-- `, `|   `).
    Ascii,
}

/// Configuration settings governing directory tree visualization and formatting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormatConfig {
    /// Visual line rendering style (Unicode vs ASCII).
    pub style: TreeStyle,
    /// Maximum nesting depth to render (root entries have depth 0). `None` renders all levels.
    pub max_depth: Option<usize>,
    /// Whether to append human-readable byte sizes to file entries (e.g. `[1.2 KB]`).
    pub include_sizes: bool,
    /// Whether to append file category tags (e.g. `[source]`, `[doc]`, `[manifest]`).
    pub include_categories: bool,
    /// Whether to append detected programming language tags (e.g. `(rust)`, `(typescript)`).
    pub include_languages: bool,
    /// Whether directories should be sorted before individual files within each level.
    pub dirs_first: bool,
    /// Maximum total entries to render before truncating with an omission note.
    pub max_entries: Option<usize>,
}

impl Default for TreeFormatConfig {
    fn default() -> Self {
        Self {
            style: TreeStyle::Unicode,
            max_depth: None,
            include_sizes: true,
            include_categories: true,
            include_languages: false,
            dirs_first: true,
            max_entries: Some(5_000),
        }
    }
}

impl TreeFormatConfig {
    /// Creates a new default `TreeFormatConfig`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the visual line drawing style.
    #[must_use]
    pub const fn with_style(mut self, style: TreeStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the maximum nesting depth ceiling.
    #[must_use]
    pub const fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Configures whether file sizes are displayed.
    #[must_use]
    pub const fn with_include_sizes(mut self, include: bool) -> Self {
        self.include_sizes = include;
        self
    }

    /// Configures whether category tags are displayed.
    #[must_use]
    pub const fn with_include_categories(mut self, include: bool) -> Self {
        self.include_categories = include;
        self
    }

    /// Configures whether language tags are displayed.
    #[must_use]
    pub const fn with_include_languages(mut self, include: bool) -> Self {
        self.include_languages = include;
        self
    }

    /// Configures whether directories are listed ahead of files.
    #[must_use]
    pub const fn with_dirs_first(mut self, dirs_first: bool) -> Self {
        self.dirs_first = dirs_first;
        self
    }

    /// Sets the maximum total rendered entries ceiling.
    #[must_use]
    pub const fn with_max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = Some(max_entries);
        self
    }
}

/// A node within the hierarchical repository directory tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoTreeNode {
    /// File or directory base name.
    pub name: String,
    /// Full normalized relative path from repository root.
    pub path: String,
    /// Whether this node represents a directory.
    pub is_dir: bool,
    /// POSIX file mode if known.
    pub mode: Option<GitFileMode>,
    /// File byte size if known.
    pub size: Option<u64>,
    /// Evaluated file category, if classified.
    pub category: Option<FileCategory>,
    /// Detected language or syntax format, if identified.
    pub language: Option<String>,
    /// Child nodes keyed by name.
    pub children: BTreeMap<String, RepoTreeNode>,
}

impl RepoTreeNode {
    /// Creates a new directory node.
    #[must_use]
    pub fn new_dir(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir: true,
            mode: Some(GitFileMode::Subdirectory),
            size: None,
            category: None,
            language: None,
            children: BTreeMap::new(),
        }
    }

    /// Creates a new file node.
    #[must_use]
    pub fn new_file(
        name: impl Into<String>,
        path: impl Into<String>,
        mode: Option<GitFileMode>,
        size: Option<u64>,
        category: Option<FileCategory>,
        language: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir: false,
            mode,
            size,
            category,
            language,
            children: BTreeMap::new(),
        }
    }

    /// Recursively counts all nested file (leaf) nodes.
    #[must_use]
    pub fn file_count(&self) -> usize {
        if !self.is_dir {
            return 1;
        }
        self.children.values().map(RepoTreeNode::file_count).sum()
    }

    /// Recursively counts all nested directory nodes.
    #[must_use]
    pub fn dir_count(&self) -> usize {
        let current = if self.is_dir && !self.path.is_empty() {
            1
        } else {
            0
        };
        let nested: usize = self
            .children
            .values()
            .filter(|c| c.is_dir)
            .map(RepoTreeNode::dir_count)
            .sum();
        current + nested
    }

    /// Recursively computes aggregate byte size of all contained files.
    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        if !self.is_dir {
            return self.size.unwrap_or(0);
        }
        self.children.values().map(RepoTreeNode::total_bytes).sum()
    }

    /// Computes the maximum directory nesting depth below this node.
    #[must_use]
    pub fn max_depth(&self) -> usize {
        if self.children.is_empty() {
            return 0;
        }
        1 + self
            .children
            .values()
            .map(RepoTreeNode::max_depth)
            .max()
            .unwrap_or(0)
    }

    /// Inserts a file entry into the tree hierarchy by path segments.
    pub fn insert_entry(
        &mut self,
        segments: &[&str],
        full_path: &str,
        mode: Option<GitFileMode>,
        size: Option<u64>,
        category: Option<FileCategory>,
        language: Option<String>,
    ) {
        if segments.is_empty() {
            return;
        }

        let first = segments[0];
        if segments.len() == 1 {
            // Leaf file
            self.children.insert(
                first.to_string(),
                Self::new_file(first, full_path, mode, size, category, language),
            );
        } else {
            // Subdirectory
            let sub_path = if self.path.is_empty() {
                first.to_string()
            } else {
                format!("{}/{}", self.path, first)
            };

            let entry = self
                .children
                .entry(first.to_string())
                .or_insert_with(|| Self::new_dir(first, sub_path));
            entry.insert_entry(&segments[1..], full_path, mode, size, category, language);
        }
    }
}

/// Complete hierarchical repository directory tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoTree {
    /// Root node of the repository tree.
    pub root: RepoTreeNode,
    /// Total count of files discovered.
    pub total_files: usize,
    /// Total count of directories discovered.
    pub total_directories: usize,
    /// Total byte size across all files.
    pub total_bytes: u64,
    /// Maximum nesting depth in the repository.
    pub max_depth: usize,
    /// Count breakdown of files by category.
    pub category_counts: BTreeMap<String, usize>,
}

impl RepoTree {
    /// Builds a `RepoTree` from an array of `GitTreeEntry` items.
    #[must_use]
    pub fn from_entries(entries: &[GitTreeEntry]) -> Self {
        let mut root = RepoTreeNode::new_dir(".", "");
        let mut total_files = 0;
        let mut total_bytes = 0;
        let mut max_depth = 0;

        for entry in entries {
            if entry.is_blob() {
                total_files += 1;
                total_bytes += entry.size.unwrap_or(0);
                let depth = entry.depth();
                if depth > max_depth {
                    max_depth = depth;
                }

                let segments: Vec<&str> = entry.path.split('/').filter(|s| !s.is_empty()).collect();
                root.insert_entry(
                    &segments,
                    &entry.path,
                    Some(entry.mode.clone()),
                    entry.size,
                    None,
                    None,
                );
            }
        }

        let total_directories = root.dir_count();

        Self {
            root,
            total_files,
            total_directories,
            total_bytes,
            max_depth,
            category_counts: BTreeMap::new(),
        }
    }

    /// Builds a `RepoTree` from an array of classified repository files.
    #[must_use]
    pub fn from_classified_files(files: &[ClassifiedFile]) -> Self {
        let mut root = RepoTreeNode::new_dir(".", "");
        let mut total_files = 0;
        let mut total_bytes = 0;
        let mut max_depth = 0;
        let mut category_counts = BTreeMap::new();

        for file in files {
            total_files += 1;
            total_bytes += file.size.unwrap_or(0);
            let depth = file.path.chars().filter(|&c| c == '/').count();
            if depth > max_depth {
                max_depth = depth;
            }

            *category_counts
                .entry(file.category.as_str().to_string())
                .or_insert(0) += 1;

            let segments: Vec<&str> = file.path.split('/').filter(|s| !s.is_empty()).collect();
            root.insert_entry(
                &segments,
                &file.path,
                Some(file.mode.clone()),
                file.size,
                Some(file.category),
                file.language.clone(),
            );
        }

        let total_directories = root.dir_count();

        Self {
            root,
            total_files,
            total_directories,
            total_bytes,
            max_depth,
            category_counts,
        }
    }

    /// Builds a `RepoTree` from a `GitTreeResponse`.
    #[must_use]
    pub fn from_tree_response(response: &GitTreeResponse) -> Self {
        Self::from_entries(&response.tree)
    }

    /// Renders the tree into a structured ASCII or Unicode text diagram according to `config`.
    #[must_use]
    pub fn render(&self, config: &TreeFormatConfig) -> String {
        let mut out = String::new();
        out.push_str(".\n");

        let mut rendered_count = 0;
        let mut truncated = false;

        self.render_node(
            &self.root,
            "",
            0,
            config,
            &mut out,
            &mut rendered_count,
            &mut truncated,
        );

        if truncated {
            out.push_str("\n... [Additional entries omitted by max_entries ceiling]\n");
        }

        // Summary footer
        let bytes_str = format_byte_size(self.total_bytes);
        out.push_str(&format!(
            "\n{} directories, {} files ({bytes_str} total)\n",
            self.total_directories, self.total_files
        ));

        out
    }

    /// Renders the tree using default configuration settings.
    #[must_use]
    pub fn render_default(&self) -> String {
        self.render(&TreeFormatConfig::default())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_node(
        &self,
        node: &RepoTreeNode,
        prefix: &str,
        depth: usize,
        config: &TreeFormatConfig,
        out: &mut String,
        rendered_count: &mut usize,
        truncated: &mut bool,
    ) {
        if let Some(max_e) = config.max_entries {
            if *rendered_count >= max_e {
                *truncated = true;
                return;
            }
        }

        if let Some(max_d) = config.max_depth {
            if depth > max_d {
                return;
            }
        }

        // Collect and sort children
        let mut children: Vec<&RepoTreeNode> = node.children.values().collect();
        if config.dirs_first {
            children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });
        } else {
            children.sort_by_key(|a| a.name.to_lowercase());
        }

        let total = children.len();
        for (i, child) in children.iter().enumerate() {
            if let Some(max_e) = config.max_entries {
                if *rendered_count >= max_e {
                    *truncated = true;
                    return;
                }
            }

            *rendered_count += 1;
            let is_last = i == total - 1;

            let (branch, next_prefix) = match config.style {
                TreeStyle::Unicode => {
                    if is_last {
                        ("└── ", format!("{prefix}    "))
                    } else {
                        ("├── ", format!("{prefix}│   "))
                    }
                }
                TreeStyle::Ascii => {
                    if is_last {
                        ("\\-- ", format!("{prefix}    "))
                    } else {
                        ("|-- ", format!("{prefix}|   "))
                    }
                }
            };

            out.push_str(prefix);
            out.push_str(branch);
            out.push_str(&child.name);

            if child.is_dir {
                out.push('/');
                out.push('\n');
                self.render_node(
                    child,
                    &next_prefix,
                    depth + 1,
                    config,
                    out,
                    rendered_count,
                    truncated,
                );
            } else {
                // Annotations
                if config.include_categories {
                    if let Some(cat) = child.category {
                        out.push_str(&format!(" [{}]", category_badge(cat)));
                    }
                }
                if config.include_languages {
                    if let Some(ref lang) = child.language {
                        out.push_str(&format!(" ({lang})"));
                    }
                }
                if config.include_sizes {
                    if let Some(sz) = child.size {
                        out.push_str(&format!(" ({})", format_byte_size(sz)));
                    }
                }
                out.push('\n');
            }
        }
    }

    /// Converts this repository tree into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(
        &self,
        section_id: impl Into<String>,
        source_url: impl Into<String>,
        config: &TreeFormatConfig,
    ) -> CdmSection {
        let rendered = self.render(config);
        let line_count = rendered.lines().count().max(1) as u32;
        let byte_size = rendered.len() as u64;
        let blake3_hash = compute_blake3_hash(rendered.as_bytes());
        let token_metrics = SectionTokenMetrics::compute_baseline(&rendered);

        let mut section =
            CdmSectionBuilder::new(section_id, source_url, SectionKind::Manifest, &rendered)
                .title("Repository Directory Tree")
                .original_path("TREE.md")
                .line_range(1, line_count)
                .byte_range(0, byte_size)
                .language("text")
                .content_hash(&blake3_hash)
                .token_metrics(token_metrics)
                .build();

        section
            .attributes
            .insert("manifest_role".to_string(), "repository_tree".to_string());
        section
            .attributes
            .insert("total_files".to_string(), self.total_files.to_string());
        section.attributes.insert(
            "total_directories".to_string(),
            self.total_directories.to_string(),
        );
        section
            .attributes
            .insert("total_bytes".to_string(), self.total_bytes.to_string());
        section
            .attributes
            .insert("max_depth".to_string(), self.max_depth.to_string());

        section
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_byte_size() {
        assert_eq!(format_byte_size(0), "0 B");
        assert_eq!(format_byte_size(512), "512 B");
        assert_eq!(format_byte_size(1024), "1.0 KB");
        assert_eq!(format_byte_size(1536), "1.5 KB");
        assert_eq!(format_byte_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_byte_size(5 * 1024 * 1024 * 1024), "5.00 GB");
    }

    #[test]
    fn test_tree_construction_and_metrics() {
        let entries = vec![
            GitTreeEntry::new(
                "Cargo.toml",
                GitFileMode::RegularFile,
                crate::tree::GitTreeEntryType::Blob,
                "1111111111111111111111111111111111111111",
                Some(250),
            ),
            GitTreeEntry::new(
                "README.md",
                GitFileMode::RegularFile,
                crate::tree::GitTreeEntryType::Blob,
                "2222222222222222222222222222222222222222",
                Some(1200),
            ),
            GitTreeEntry::new(
                "src/lib.rs",
                GitFileMode::RegularFile,
                crate::tree::GitTreeEntryType::Blob,
                "3333333333333333333333333333333333333333",
                Some(4500),
            ),
            GitTreeEntry::new(
                "src/parser.rs",
                GitFileMode::RegularFile,
                crate::tree::GitTreeEntryType::Blob,
                "4444444444444444444444444444444444444444",
                Some(800),
            ),
            GitTreeEntry::new(
                "tests/integration_test.rs",
                GitFileMode::RegularFile,
                crate::tree::GitTreeEntryType::Blob,
                "5555555555555555555555555555555555555555",
                Some(2100),
            ),
        ];

        let tree = RepoTree::from_entries(&entries);
        assert_eq!(tree.total_files, 5);
        assert_eq!(tree.total_directories, 2); // src, tests
        assert_eq!(tree.total_bytes, 250 + 1200 + 4500 + 800 + 2100);
        assert_eq!(tree.max_depth, 1);

        let rendered = tree.render_default();
        assert!(rendered.starts_with(".\n"));
        assert!(rendered.contains("src/"));
        assert!(rendered.contains("lib.rs"));
        assert!(rendered.contains("README.md"));
        assert!(rendered.contains("2 directories, 5 files"));
    }

    #[test]
    fn test_tree_ascii_style_rendering() {
        let entries = vec![
            GitTreeEntry::new(
                "Makefile",
                GitFileMode::RegularFile,
                crate::tree::GitTreeEntryType::Blob,
                "1111111111111111111111111111111111111111",
                Some(100),
            ),
            GitTreeEntry::new(
                "bin/run.sh",
                GitFileMode::ExecutableFile,
                crate::tree::GitTreeEntryType::Blob,
                "2222222222222222222222222222222222222222",
                Some(300),
            ),
        ];

        let tree = RepoTree::from_entries(&entries);
        let config = TreeFormatConfig::new().with_style(TreeStyle::Ascii);
        let rendered = tree.render(&config);

        assert!(rendered.contains("|-- ") || rendered.contains("\\-- "));
        assert!(!rendered.contains("├── "));
    }

    #[test]
    fn test_tree_to_cdm_section() {
        let files = vec![
            ClassifiedFile {
                path: "src/main.rs".to_string(),
                category: FileCategory::SourceCode,
                mode: GitFileMode::RegularFile,
                sha: "1111111111111111111111111111111111111111".to_string(),
                size: Some(1500),
                language: Some("rust".to_string()),
                is_eligible: true,
                exclusion_reason: None,
            },
            ClassifiedFile {
                path: "Cargo.toml".to_string(),
                category: FileCategory::DependencyManifest,
                mode: GitFileMode::RegularFile,
                sha: "2222222222222222222222222222222222222222".to_string(),
                size: Some(400),
                language: Some("toml".to_string()),
                is_eligible: true,
                exclusion_reason: None,
            },
        ];

        let tree = RepoTree::from_classified_files(&files);
        assert_eq!(tree.category_counts.get("source_code"), Some(&1));
        assert_eq!(tree.category_counts.get("dependency_manifest"), Some(&1));

        let config = TreeFormatConfig::default();
        let section = tree.to_cdm_section("sec-tree", "https://github.com/org/repo", &config);

        assert_eq!(section.section_id, "sec-tree");
        assert_eq!(section.kind, SectionKind::Manifest);
        assert_eq!(
            section.attributes.get("manifest_role"),
            Some(&"repository_tree".to_string())
        );
        assert_eq!(
            section.attributes.get("total_files"),
            Some(&"2".to_string())
        );
        assert_eq!(
            section.attributes.get("total_directories"),
            Some(&"1".to_string())
        );
    }
}
