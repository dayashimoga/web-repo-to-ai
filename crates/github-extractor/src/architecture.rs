//! # Repository Architecture Context & System Overview
//!
//! Provides automated architecture analysis, workspace topology classification,
//! entry point discovery, component cataloging, technology stack inference,
//! architecture document reference mapping, and Canonical Context Document Model
//! ([`CdmSection`]) overview generation.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmSection, CdmSectionBuilder, SectionKind, SectionTokenMetrics};

use crate::classification::{ClassifiedFile, FileCategory, TreeClassificationReport};
use crate::config_test::ConfigTestExtractionReport;
use crate::dependency::DependencyExtractionReport;
use crate::documentation::DocExtractionReport;
use crate::manifest::RepoFileManifest;
use crate::metadata::GitHubRepositoryMetadata;
use crate::source::compute_blake3_hash;

/// Inferred repository workspace structure and module topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceType {
    /// Single standalone application or package.
    SinglePackage,
    /// Multi-crate or multi-package monorepo containing distinct modular subsystems.
    Monorepo,
    /// Distributed microservices architecture with independent service directories.
    MultiService,
    /// Standalone reusable software library or framework crate.
    Library,
    /// Other or uncategorized workspace layout.
    Other,
}

impl WorkspaceType {
    /// Returns the snake_case string identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SinglePackage => "single_package",
            Self::Monorepo => "monorepo",
            Self::MultiService => "multi_service",
            Self::Library => "library",
            Self::Other => "other",
        }
    }

    /// Returns `true` if the workspace exhibits a multi-module or monorepo structure.
    #[must_use]
    pub const fn is_monorepo(self) -> bool {
        matches!(self, Self::Monorepo | Self::MultiService)
    }
}

/// The architectural role of an executable or library entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPointRole {
    /// Application executable entry point (e.g. `main.rs`, `main.go`, `index.ts`).
    BinaryMain,
    /// Library root interface export (e.g. `lib.rs`, `index.ts`, `mod.rs`).
    LibraryRoot,
    /// HTTP/REST/gRPC web API endpoint handler or router.
    WebApi,
    /// Frontend web application route or page view (e.g. `+page.svelte`, `page.tsx`).
    UiRoute,
    /// Background worker, queue consumer, or daemon runner.
    WorkerDaemon,
    /// Automation or CLI maintenance script.
    ScriptCli,
}

impl EntryPointRole {
    /// Returns a human-readable identifier for display.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BinaryMain => "binary_main",
            Self::LibraryRoot => "library_root",
            Self::WebApi => "web_api",
            Self::UiRoute => "ui_route",
            Self::WorkerDaemon => "worker_daemon",
            Self::ScriptCli => "script_cli",
        }
    }
}

/// Discovered application or subsystem entry point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointInfo {
    /// Relative repository path to the entry point file.
    pub path: String,
    /// Detected programming language or syntax format.
    pub language: Option<String>,
    /// Structural role of this entry point.
    pub role: EntryPointRole,
    /// Functional description or context note.
    pub description: String,
}

impl EntryPointInfo {
    /// Creates a new entry point record.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        language: Option<impl Into<String>>,
        role: EntryPointRole,
        description: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            language: language.map(Into::into),
            role,
            description: description.into(),
        }
    }
}

/// The functional role of a repository component or subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentRole {
    /// Backend HTTP, REST, or RPC service.
    Service,
    /// Reusable library crate or shared package.
    LibraryCrate,
    /// User-facing web frontend application.
    WebFrontend,
    /// Background worker or processing daemon.
    Worker,
    /// Operational CLI or automation script suite.
    ToolCli,
    /// Infrastructure, deployment configuration, or container definitions.
    Infrastructure,
    /// Technical documentation, specifications, or architectural records.
    Documentation,
    /// General repository module.
    GeneralModule,
}

impl ComponentRole {
    /// Returns the snake_case string identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::LibraryCrate => "library_crate",
            Self::WebFrontend => "web_frontend",
            Self::Worker => "worker",
            Self::ToolCli => "tool_cli",
            Self::Infrastructure => "infrastructure",
            Self::Documentation => "documentation",
            Self::GeneralModule => "general_module",
        }
    }
}

/// High-level catalog entry for a discovered subsystem or package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInfo {
    /// Identifier name of the component (e.g. `crates/cdm`, `apps/web`).
    pub name: String,
    /// Relative root path in the repository.
    pub root_path: String,
    /// Functional category role.
    pub role: ComponentRole,
    /// Primary programming language or technology, if determined.
    pub primary_language: Option<String>,
    /// Number of files discovered in this component.
    pub file_count: usize,
    /// Aggregate byte size of component files.
    pub total_bytes: u64,
    /// High-level description of the component's purpose.
    pub description: String,
}

/// Consolidated repository architecture context and system overview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoArchitectureContext {
    /// Inferred workspace structure / module topology.
    pub workspace_type: WorkspaceType,
    /// Primary programming languages detected with file counts.
    pub primary_languages: BTreeMap<String, usize>,
    /// Package ecosystems discovered (e.g. Cargo, Npm, Pip, GoMod).
    pub ecosystems: Vec<String>,
    /// Key technologies, frameworks, and runtime libraries detected.
    pub key_technologies: Vec<String>,
    /// Discovered application and service entry points.
    pub entry_points: Vec<EntryPointInfo>,
    /// Catalog of discovered components, crates, or packages.
    pub components: Vec<ComponentInfo>,
    /// Discovered architecture, design, and specification documents.
    pub architecture_docs: Vec<String>,
    /// Discovered build and CI/CD pipelines.
    pub ci_cd_pipelines: Vec<String>,
    /// Concise summary narrative of the repository architecture.
    pub summary: String,
}

impl RepoArchitectureContext {
    /// Synthesizes a comprehensive `RepoArchitectureContext` from repository metadata,
    /// file classification, file manifest, and extraction reports.
    #[must_use]
    pub fn from_repository_data(
        classification: &TreeClassificationReport,
        manifest: Option<&RepoFileManifest>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
        metadata: Option<&GitHubRepositoryMetadata>,
    ) -> Self {
        let files = &classification.files;

        // 1. Primary languages
        let mut primary_languages = BTreeMap::new();
        for file in files {
            if let Some(ref lang) = file.language {
                *primary_languages.entry(lang.clone()).or_insert(0) += 1;
            }
        }

        // 2. Discover ecosystems
        let mut ecosystems_set = BTreeSet::new();
        if let Some(dep_report) = dependencies {
            for dep in &dep_report.extracted {
                ecosystems_set.insert(dep.ecosystem.as_str().to_string());
            }
        } else {
            for file in files {
                let name = file.file_name().to_ascii_lowercase();
                if name == "cargo.toml" {
                    ecosystems_set.insert("cargo".to_string());
                } else if name == "package.json" {
                    ecosystems_set.insert("npm".to_string());
                } else if name == "requirements.txt" || name == "pyproject.toml" {
                    ecosystems_set.insert("pip".to_string());
                } else if name == "go.mod" {
                    ecosystems_set.insert("go_mod".to_string());
                }
            }
        }
        let ecosystems: Vec<String> = ecosystems_set.into_iter().collect();

        // 3. Workspace Type Invariant Detection
        let workspace_type = Self::detect_workspace_type(files);

        // 4. Discover Entry Points
        let entry_points = Self::detect_entry_points(files);

        // 5. Catalog Components
        let components = Self::catalog_components(files, workspace_type);

        // 6. Discover Architecture and Design Documents
        let architecture_docs = Self::discover_architecture_docs(files, docs);

        // 7. Discover CI/CD Pipelines
        let ci_cd_pipelines = Self::discover_ci_cd_pipelines(files, configs_tests);

        // 8. Key Technologies and Frameworks
        let key_technologies =
            Self::infer_key_technologies(files, dependencies, configs_tests, metadata);

        // 9. Synthesize Narrative Summary
        let summary = Self::synthesize_summary(
            workspace_type,
            &ecosystems,
            &key_technologies,
            components.len(),
            entry_points.len(),
            manifest.map(|m| m.total_files).unwrap_or(files.len()),
            metadata,
        );

        Self {
            workspace_type,
            primary_languages,
            ecosystems,
            key_technologies,
            entry_points,
            components,
            architecture_docs,
            ci_cd_pipelines,
            summary,
        }
    }

    /// Detects the workspace layout topology based on directory conventions and manifests.
    fn detect_workspace_type(files: &[ClassifiedFile]) -> WorkspaceType {
        let mut has_crates_dir = false;
        let mut has_services_dir = false;
        let mut has_apps_dir = false;
        let mut has_packages_dir = false;
        let mut root_manifest_count = 0;
        let mut nested_manifest_count = 0;

        for file in files {
            let path = &file.path;
            if path.starts_with("crates/") {
                has_crates_dir = true;
            } else if path.starts_with("services/") {
                has_services_dir = true;
            } else if path.starts_with("apps/") {
                has_apps_dir = true;
            } else if path.starts_with("packages/") {
                has_packages_dir = true;
            }

            let name = file.file_name().to_ascii_lowercase();
            if name == "cargo.toml" || name == "package.json" || name == "go.mod" {
                if path.contains('/') {
                    nested_manifest_count += 1;
                } else {
                    root_manifest_count += 1;
                }
            }
        }

        if (has_services_dir && (has_apps_dir || has_crates_dir))
            || has_crates_dir
            || has_packages_dir
            || nested_manifest_count >= 2
        {
            WorkspaceType::Monorepo
        } else if has_services_dir {
            WorkspaceType::MultiService
        } else {
            // Check if it's primarily a library
            let has_main = files.iter().any(|f| {
                f.path == "src/main.rs" || f.path == "main.go" || f.path == "src/index.ts"
            });
            let has_lib = files
                .iter()
                .any(|f| f.path == "src/lib.rs" || f.path.ends_with("/lib.rs"));

            if has_lib && !has_main {
                WorkspaceType::Library
            } else if root_manifest_count >= 1 || has_main {
                WorkspaceType::SinglePackage
            } else {
                WorkspaceType::Other
            }
        }
    }

    /// Scans repository paths for executable, service, and library entry points.
    fn detect_entry_points(files: &[ClassifiedFile]) -> Vec<EntryPointInfo> {
        let mut entries = Vec::new();

        for file in files {
            let path = &file.path;
            let lower = path.to_ascii_lowercase();

            // 1. Rust Mains & Libs
            if lower.ends_with("/src/main.rs") || lower == "src/main.rs" {
                let (role, desc) = if lower.contains("services/api") || lower.contains("api/") {
                    (EntryPointRole::WebApi, "HTTP API service entry point")
                } else if lower.contains("services/worker")
                    || lower.contains("worker/")
                    || lower.contains("daemon")
                {
                    (
                        EntryPointRole::WorkerDaemon,
                        "Background worker daemon entry point",
                    )
                } else {
                    (
                        EntryPointRole::BinaryMain,
                        "Executable application main entry point",
                    )
                };
                entries.push(EntryPointInfo::new(
                    path,
                    file.language.as_ref(),
                    role,
                    desc,
                ));
            } else if lower == "src/lib.rs" || lower.ends_with("/src/lib.rs") {
                entries.push(EntryPointInfo::new(
                    path,
                    file.language.as_ref(),
                    EntryPointRole::LibraryRoot,
                    "Library root module and public export API",
                ));
            }
            // 2. Web UI Routes
            else if lower.ends_with("/src/routes/+page.svelte")
                || lower == "src/routes/+page.svelte"
            {
                entries.push(EntryPointInfo::new(
                    path,
                    file.language.as_ref(),
                    EntryPointRole::UiRoute,
                    "SvelteKit root page view and interactive UI route",
                ));
            } else if lower.ends_with("/app/page.tsx") || lower.ends_with("/pages/index.tsx") {
                entries.push(EntryPointInfo::new(
                    path,
                    file.language.as_ref(),
                    EntryPointRole::UiRoute,
                    "Web frontend root application route",
                ));
            }
            // 3. Go / Node Mains
            else if lower == "main.go" || lower.ends_with("/main.go") {
                entries.push(EntryPointInfo::new(
                    path,
                    file.language.as_ref(),
                    EntryPointRole::BinaryMain,
                    "Go executable application main driver",
                ));
            } else if lower == "src/index.ts" || lower == "index.ts" || lower == "src/index.js" {
                entries.push(EntryPointInfo::new(
                    path,
                    file.language.as_ref(),
                    EntryPointRole::BinaryMain,
                    "JavaScript/TypeScript runtime entry point",
                ));
            }
            // 4. Scripts & Automation
            else if (lower.starts_with("scripts/") || lower.starts_with("bin/"))
                && (lower.ends_with(".ps1") || lower.ends_with(".sh") || lower.ends_with(".py"))
                && (lower.contains("check-all")
                    || lower.contains("dev-up")
                    || lower.contains("dev-"))
            {
                entries.push(EntryPointInfo::new(
                    path,
                    file.language.as_ref(),
                    EntryPointRole::ScriptCli,
                    "Development and verification orchestration script",
                ));
            }
        }

        // Deterministic sort by path
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        entries
    }

    /// Catalogs major components based on directory hierarchy groupings.
    fn catalog_components(
        files: &[ClassifiedFile],
        workspace_type: WorkspaceType,
    ) -> Vec<ComponentInfo> {
        let mut component_groups: BTreeMap<String, (ComponentRole, Vec<&ClassifiedFile>)> =
            BTreeMap::new();

        for file in files {
            let path = &file.path;
            let parts: Vec<&str> = path.split('/').collect();

            if parts.len() >= 2 {
                let first = parts[0];
                let second = parts[1];

                if first == "crates" {
                    let comp_path = format!("crates/{second}");
                    let entry = component_groups
                        .entry(comp_path)
                        .or_insert_with(|| (ComponentRole::LibraryCrate, Vec::new()));
                    entry.1.push(file);
                    continue;
                } else if first == "services" {
                    let comp_path = format!("services/{second}");
                    let role = if second.contains("api") {
                        ComponentRole::Service
                    } else if second.contains("worker") {
                        ComponentRole::Worker
                    } else {
                        ComponentRole::Service
                    };
                    let entry = component_groups
                        .entry(comp_path)
                        .or_insert_with(|| (role, Vec::new()));
                    entry.1.push(file);
                    continue;
                } else if first == "apps" {
                    let comp_path = format!("apps/{second}");
                    let entry = component_groups
                        .entry(comp_path)
                        .or_insert_with(|| (ComponentRole::WebFrontend, Vec::new()));
                    entry.1.push(file);
                    continue;
                } else if first == "packages" {
                    let comp_path = format!("packages/{second}");
                    let entry = component_groups
                        .entry(comp_path)
                        .or_insert_with(|| (ComponentRole::LibraryCrate, Vec::new()));
                    entry.1.push(file);
                    continue;
                }
            }

            // Top-level groupings
            let top_dir = parts.first().copied().unwrap_or("");
            if top_dir == "deploy" {
                let entry = component_groups
                    .entry("deploy".to_string())
                    .or_insert_with(|| (ComponentRole::Infrastructure, Vec::new()));
                entry.1.push(file);
            } else if top_dir == "docs" {
                let entry = component_groups
                    .entry("docs".to_string())
                    .or_insert_with(|| (ComponentRole::Documentation, Vec::new()));
                entry.1.push(file);
            } else if top_dir == "scripts" {
                let entry = component_groups
                    .entry("scripts".to_string())
                    .or_insert_with(|| (ComponentRole::ToolCli, Vec::new()));
                entry.1.push(file);
            } else if (workspace_type == WorkspaceType::SinglePackage
                || workspace_type == WorkspaceType::Library)
                && top_dir == "src"
            {
                let role = if workspace_type == WorkspaceType::Library {
                    ComponentRole::LibraryCrate
                } else {
                    ComponentRole::Service
                };
                let entry = component_groups
                    .entry("src".to_string())
                    .or_insert_with(|| (role, Vec::new()));
                entry.1.push(file);
            }
        }

        let mut catalog = Vec::new();
        for (root_path, (role, comp_files)) in component_groups {
            let file_count = comp_files.len();
            let total_bytes: u64 = comp_files.iter().filter_map(|f| f.size).sum();

            // Determine primary language in component
            let mut lang_counts = BTreeMap::new();
            for f in &comp_files {
                if let Some(ref l) = f.language {
                    *lang_counts.entry(l.clone()).or_insert(0) += 1;
                }
            }
            let primary_language = lang_counts
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(lang, _)| lang);

            let name = root_path
                .rsplit('/')
                .next()
                .unwrap_or(&root_path)
                .to_string();
            let description = match role {
                ComponentRole::Service => {
                    format!("Backend HTTP/RPC service hosted in `{root_path}`")
                }
                ComponentRole::LibraryCrate => {
                    format!("Shared modular library crate located in `{root_path}`")
                }
                ComponentRole::WebFrontend => {
                    format!("Interactive web frontend application located in `{root_path}`")
                }
                ComponentRole::Worker => {
                    format!("Background queue and processing worker in `{root_path}`")
                }
                ComponentRole::ToolCli => {
                    format!("Operational tooling and developer scripts in `{root_path}`")
                }
                ComponentRole::Infrastructure => {
                    format!("Container, deployment, and orchestration specs in `{root_path}`")
                }
                ComponentRole::Documentation => {
                    format!(
                        "Architecture, technical guides, and API specifications in `{root_path}`"
                    )
                }
                ComponentRole::GeneralModule => {
                    format!("General subsystem module in `{root_path}`")
                }
            };

            catalog.push(ComponentInfo {
                name,
                root_path,
                role,
                primary_language,
                file_count,
                total_bytes,
                description,
            });
        }

        catalog.sort_by(|a, b| a.root_path.cmp(&b.root_path));
        catalog
    }

    /// Locates key architecture, design, and specification documents.
    fn discover_architecture_docs(
        files: &[ClassifiedFile],
        docs: Option<&DocExtractionReport>,
    ) -> Vec<String> {
        let mut results = BTreeSet::new();

        let doc_keywords = [
            "architecture",
            "data_model",
            "decision",
            "adr",
            "design",
            "requirement",
            "contract",
            "specification",
            "security",
        ];

        for file in files {
            if file.category == FileCategory::Documentation {
                let lower = file.path.to_ascii_lowercase();
                for kw in &doc_keywords {
                    if lower.contains(kw) {
                        results.insert(file.path.clone());
                        break;
                    }
                }
            }
        }

        if let Some(doc_report) = docs {
            for doc in &doc_report.extracted {
                let lower = doc.path.to_ascii_lowercase();
                for kw in &doc_keywords {
                    if lower.contains(kw) {
                        results.insert(doc.path.clone());
                        break;
                    }
                }
            }
        }

        results.into_iter().collect()
    }

    /// Discovers active CI/CD and automation workflow pipelines.
    fn discover_ci_cd_pipelines(
        files: &[ClassifiedFile],
        configs: Option<&ConfigTestExtractionReport>,
    ) -> Vec<String> {
        let mut pipelines = BTreeSet::new();

        for file in files {
            let lower = file.path.to_ascii_lowercase();
            if (lower.starts_with(".github/workflows/")
                && (lower.ends_with(".yml") || lower.ends_with(".yaml")))
                || lower == "dockerfile"
                || (lower.starts_with("deploy/") && lower.contains("compose"))
            {
                pipelines.insert(file.path.clone());
            }
        }

        if let Some(cfg_report) = configs {
            for cfg in &cfg_report.extracted_configs {
                let lower = cfg.path.to_ascii_lowercase();
                if lower.starts_with(".github/workflows/") {
                    pipelines.insert(cfg.path.clone());
                }
            }
        }

        pipelines.into_iter().collect()
    }

    /// Infers key runtime frameworks, technologies, and tooling.
    fn infer_key_technologies(
        files: &[ClassifiedFile],
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
        metadata: Option<&GitHubRepositoryMetadata>,
    ) -> Vec<String> {
        let mut tech = BTreeSet::new();

        // 1. Check known dependency packages
        let known_libs = [
            ("tokio", "Tokio (Async Runtime)"),
            ("axum", "Axum (Web Framework)"),
            ("reqwest", "Reqwest (HTTP Client)"),
            ("sqlx", "SQLx (Async Database)"),
            ("serde", "Serde (Serialization)"),
            ("blake3", "BLAKE3 (Hashing)"),
            ("svelte", "Svelte (Reactive UI)"),
            ("@sveltejs/kit", "SvelteKit (Full-Stack Frontend)"),
            ("tailwindcss", "Tailwind CSS (Styling)"),
            ("playwright", "Playwright (Headless Browser)"),
            ("vitest", "Vitest (Test Runner)"),
            ("typescript", "TypeScript (Type Checker)"),
            ("react", "React"),
            ("vue", "Vue"),
            ("fastapi", "FastAPI"),
            ("django", "Django"),
            ("gin-gonic", "Gin Web Framework"),
        ];

        if let Some(dep_report) = dependencies {
            for dep_file in &dep_report.extracted {
                if let Some(ref parsed) = dep_file.parsed {
                    for declared in &parsed.dependencies {
                        let name_lower = declared.name.to_ascii_lowercase();
                        for (needle, label) in &known_libs {
                            if name_lower == *needle || name_lower.contains(needle) {
                                tech.insert((*label).to_string());
                            }
                        }
                    }
                }
            }
        }

        // 2. Check configuration and test reports
        if let Some(cfg_report) = configs_tests {
            for cfg in &cfg_report.extracted_configs {
                let lower = cfg.path.to_ascii_lowercase();
                if lower.contains("eslint") {
                    tech.insert("ESLint (Linter)".to_string());
                } else if lower.contains("prettier") {
                    tech.insert("Prettier (Formatter)".to_string());
                } else if lower.contains("biome") {
                    tech.insert("Biome (Linter/Formatter)".to_string());
                } else if lower.contains("clippy") {
                    tech.insert("Clippy (Rust Linter)".to_string());
                } else if lower.contains("vitest") {
                    tech.insert("Vitest (Test Runner)".to_string());
                } else if lower.contains("playwright") {
                    tech.insert("Playwright (Browser Testing)".to_string());
                }
            }
        }

        // 3. Check build & infrastructure files
        for file in files {
            let lower = file.path.to_ascii_lowercase();
            if lower == "dockerfile" || lower.contains("podman") {
                tech.insert("Podman / Docker (Containers)".to_string());
            } else if lower.contains("compose.yml") || lower.contains("compose.yaml") {
                tech.insert("Compose (Multi-Container)".to_string());
            } else if lower.ends_with("tailwind.config.ts") || lower.ends_with("tailwind.config.js")
            {
                tech.insert("Tailwind CSS (Styling)".to_string());
            } else if lower.ends_with("vite.config.ts") || lower.ends_with("vite.config.js") {
                tech.insert("Vite (Frontend Bundler)".to_string());
            } else if lower.starts_with(".github/workflows/") {
                tech.insert("GitHub Actions (CI/CD)".to_string());
            }
        }

        // 3. Check metadata topics
        if let Some(meta) = metadata {
            for topic in &meta.topics {
                let lower = topic.to_ascii_lowercase();
                if lower == "rust" {
                    tech.insert("Rust".to_string());
                } else if lower == "typescript" {
                    tech.insert("TypeScript".to_string());
                } else if lower == "svelte" || lower == "sveltekit" {
                    tech.insert("SvelteKit (Full-Stack Frontend)".to_string());
                }
            }
        }

        tech.into_iter().collect()
    }

    /// Produces a human-readable narrative executive summary.
    fn synthesize_summary(
        workspace: WorkspaceType,
        ecosystems: &[String],
        technologies: &[String],
        component_count: usize,
        entry_point_count: usize,
        total_files: usize,
        metadata: Option<&GitHubRepositoryMetadata>,
    ) -> String {
        let repo_desc = metadata
            .and_then(|m| m.description.as_deref())
            .unwrap_or("software repository");

        let eco_str = if ecosystems.is_empty() {
            "polyglot".to_string()
        } else {
            ecosystems.join(", ")
        };

        let tech_highlight = if technologies.is_empty() {
            String::new()
        } else {
            let top_tech: Vec<&str> = technologies.iter().take(4).map(String::as_str).collect();
            format!(" Powered by {}.", top_tech.join(", "))
        };

        format!(
            "Structured as a {} across {} discovered files with {} distinct architectural components and {} entry points. Built in the {} ecosystem.{}{}",
            workspace.as_str(),
            total_files,
            component_count,
            entry_point_count,
            eco_str,
            tech_highlight,
            if !repo_desc.is_empty() {
                format!(" Context: {repo_desc}.")
            } else {
                String::new()
            }
        )
    }

    /// Formats the architecture context into a structured GitHub-Flavored Markdown document.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Repository Architecture & System Overview\n\n");

        out.push_str("## Executive Summary\n\n");
        out.push_str(&self.summary);
        out.push_str("\n\n");

        out.push_str("## System Profile\n\n");
        out.push_str("| Attribute | Value |\n");
        out.push_str("| :--- | :--- |\n");
        out.push_str(&format!(
            "| Workspace Topology | `{}` |\n",
            self.workspace_type.as_str()
        ));
        out.push_str(&format!(
            "| Package Ecosystems | {} |\n",
            if self.ecosystems.is_empty() {
                "None detected".to_string()
            } else {
                self.ecosystems.join(", ")
            }
        ));
        out.push_str(&format!(
            "| Components Cataloged | {} |\n",
            self.components.len()
        ));
        out.push_str(&format!(
            "| Entry Points Identified | {} |\n",
            self.entry_points.len()
        ));
        out.push_str(&format!(
            "| Key Architecture Docs | {} |\n",
            self.architecture_docs.len()
        ));
        out.push_str(&format!(
            "| CI/CD Pipelines | {} |\n\n",
            self.ci_cd_pipelines.len()
        ));

        // Key Technologies
        if !self.key_technologies.is_empty() {
            out.push_str("## Key Technologies & Frameworks\n\n");
            for tech in &self.key_technologies {
                out.push_str(&format!("- **{tech}**\n"));
            }
            out.push('\n');
        }

        // Subsystems & Components Table
        if !self.components.is_empty() {
            out.push_str("## Subsystems & Component Catalog\n\n");
            out.push_str("| Component | Root Path | Role | Language | Files | Description |\n");
            out.push_str("| :--- | :--- | :--- | :--- | :---: | :--- |\n");
            for comp in &self.components {
                let lang = comp.primary_language.as_deref().unwrap_or("-");
                out.push_str(&format!(
                    "| `{}` | `{}` | `{}` | {} | {} | {} |\n",
                    comp.name,
                    comp.root_path,
                    comp.role.as_str(),
                    lang,
                    comp.file_count,
                    comp.description
                ));
            }
            out.push('\n');
        }

        // Entry Points Table
        if !self.entry_points.is_empty() {
            out.push_str("## Application & Service Entry Points\n\n");
            out.push_str("| Path | Role | Language | Purpose |\n");
            out.push_str("| :--- | :--- | :--- | :--- |\n");
            for entry in &self.entry_points {
                let lang = entry.language.as_deref().unwrap_or("-");
                out.push_str(&format!(
                    "| `{}` | `{}` | {} | {} |\n",
                    entry.path,
                    entry.role.as_str(),
                    lang,
                    entry.description
                ));
            }
            out.push('\n');
        }

        // Architecture Docs
        if !self.architecture_docs.is_empty() {
            out.push_str("## Key Architectural & Design Documentation\n\n");
            for doc in &self.architecture_docs {
                out.push_str(&format!("- [`{doc}`]({doc})\n"));
            }
            out.push('\n');
        }

        // CI/CD Pipelines
        if !self.ci_cd_pipelines.is_empty() {
            out.push_str("## Build & CI/CD Pipelines\n\n");
            for pipeline in &self.ci_cd_pipelines {
                out.push_str(&format!("- `{pipeline}`\n"));
            }
            out.push('\n');
        }

        out
    }

    /// Serializes this architecture context into a pretty-printed JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serializes this architecture context into a compact single-line JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this architecture context into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(
        &self,
        section_id: impl Into<String>,
        source_url: impl Into<String>,
    ) -> CdmSection {
        let markdown = self.to_markdown();
        let line_count = markdown.lines().count().max(1) as u32;
        let byte_size = markdown.len() as u64;
        let blake3_hash = compute_blake3_hash(markdown.as_bytes());
        let token_metrics = SectionTokenMetrics::compute_baseline(&markdown);

        let mut section =
            CdmSectionBuilder::new(section_id, source_url, SectionKind::Prose, &markdown)
                .title("Repository Architecture & System Overview")
                .original_path("ARCHITECTURE_OVERVIEW.md")
                .line_range(1, line_count)
                .byte_range(0, byte_size)
                .language("markdown")
                .content_hash(&blake3_hash)
                .token_metrics(token_metrics)
                .build();

        section.attributes.insert(
            "manifest_role".to_string(),
            "architecture_context".to_string(),
        );
        section.attributes.insert(
            "workspace_type".to_string(),
            self.workspace_type.as_str().to_string(),
        );
        section.attributes.insert(
            "component_count".to_string(),
            self.components.len().to_string(),
        );
        section.attributes.insert(
            "entry_point_count".to_string(),
            self.entry_points.len().to_string(),
        );
        section.attributes.insert(
            "architecture_doc_count".to_string(),
            self.architecture_docs.len().to_string(),
        );
        section.attributes.insert(
            "pipeline_count".to_string(),
            self.ci_cd_pipelines.len().to_string(),
        );

        section
    }
}
