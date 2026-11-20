//! # AST-Boundary Code Chunking Integration Tests
//!
//! Validates multi-language syntactic splitting across Rust, Python, TypeScript, and Go.

use web_repo_to_ai_cdm::{CdmSection, LineRange, SectionKind};
use web_repo_to_ai_chunking::{
    chunk_code_section, ChunkKind, ChunkingConfig, ChunkingReport, TokenCounter,
};
use web_repo_to_ai_core::DocumentId;

#[test]
fn test_rust_ast_chunking_functions_and_structs() {
    let rust_code = r#"// Preamble: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

/// Represents a user account record in the database.
pub struct UserAccount {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub is_active: bool,
}

impl UserAccount {
    /// Creates a new active user account.
    pub fn new(id: u64, username: &str, email: &str) -> Self {
        Self {
            id,
            username: username.to_string(),
            email: email.to_string(),
            is_active: true,
        }
    }

    /// Deactivates the user account.
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

/// Validates user credentials against the database hash.
pub fn validate_credentials(user: &UserAccount, password_hash: &str) -> bool {
    !password_hash.is_empty() && user.is_active
}
"#;

    let section = CdmSection {
        section_id: "sec_rust_1".to_string(),
        parent_id: None,
        title: Some("crates/auth/src/user.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("crates/auth/src/user.rs".to_string()),
        source_url: "https://github.com/example/repo/blob/main/crates/auth/src/user.rs".to_string(),
        line_range: Some(LineRange::new(1, 36)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: CdmSection::compute_hash(rust_code),
        body: rust_code.to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(100)
        .with_max_tokens(200);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_code_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("chunking succeeds");

    assert!(!chunks.is_empty(), "must generate at least one chunk");
    assert_eq!(report.ast_chunks_count, chunks.len());

    // Verify all chunks have CodeAstDeclaration kind
    for chunk in &chunks {
        assert_eq!(chunk.kind, ChunkKind::CodeAstDeclaration);
        assert!(chunk.content.contains("// File: crates/auth/src/user.rs"));
        assert!(!chunk.content_hash.is_empty());
        assert!(chunk.token_count > 0);
    }

    // Verify symbols were extracted
    let all_symbols: Vec<_> = chunks.iter().flat_map(|c| c.ast_symbols.iter()).collect();

    let symbol_names: Vec<_> = all_symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        symbol_names.contains(&"UserAccount"),
        "must extract struct UserAccount"
    );
    assert!(symbol_names.contains(&"new"), "must extract method new");
    assert!(
        symbol_names.contains(&"validate_credentials"),
        "must extract fn validate_credentials"
    );
}

#[test]
fn test_python_ast_chunking_classes_and_functions() {
    let python_code = r#"# Standard library imports
import os
import sys
from typing import Optional, List

class DataPipeline:
    """Manages extract-transform-load data processing pipelines."""

    def __init__(self, name: str, batch_size: int = 100):
        self.name = name
        self.batch_size = batch_size
        self.records = []

    def load_data(self, source_path: str) -> int:
        """Loads records from a source file."""
        if not os.path.exists(source_path):
            raise FileNotFoundError(f"Source not found: {source_path}")
        self.records = [f"record_{i}" for i in range(self.batch_size)]
        return len(self.records)

    def process(self) -> List[str]:
        """Processes all loaded records."""
        return [r.upper() for r in self.records]

def run_pipeline(name: str, path: str) -> None:
    """Executes a named pipeline."""
    pipeline = DataPipeline(name)
    count = pipeline.load_data(path)
    print(f"Processed {count} records")
"#;

    let section = CdmSection {
        section_id: "sec_py_1".to_string(),
        parent_id: None,
        title: Some("pipeline.py".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("pipeline.py".to_string()),
        source_url: "https://github.com/example/repo/blob/main/pipeline.py".to_string(),
        line_range: Some(LineRange::new(1, 28)),
        byte_range: None,
        language: Some("python".to_string()),
        content_hash: CdmSection::compute_hash(python_code),
        body: python_code.to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(80)
        .with_max_tokens(150);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_code_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("chunking succeeds");

    assert!(!chunks.is_empty());
    let all_symbols: Vec<_> = chunks.iter().flat_map(|c| c.ast_symbols.iter()).collect();

    let names: Vec<_> = all_symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"DataPipeline"),
        "must extract class DataPipeline"
    );
    assert!(
        names.contains(&"run_pipeline"),
        "must extract function run_pipeline"
    );
}

#[test]
fn test_typescript_ast_chunking_interfaces_and_classes() {
    let ts_code = r#"import { Request, Response } from 'express';

export interface RouteHandlerConfig {
    path: string;
    method: 'GET' | 'POST' | 'PUT' | 'DELETE';
    authenticated: boolean;
}

export class ApiGateway {
    private routes: Map<string, RouteHandlerConfig> = new Map();

    public registerRoute(config: RouteHandlerConfig): void {
        const key = `${config.method}:${config.path}`;
        this.routes.set(key, config);
    }

    public handleRequest(req: Request, res: Response): void {
        const key = `${req.method}:${req.path}`;
        const route = this.routes.get(key);
        if (!route) {
            res.status(404).json({ error: 'Route not found' });
            return;
        }
        res.status(200).json({ status: 'ok' });
    }
}
"#;

    let section = CdmSection {
        section_id: "sec_ts_1".to_string(),
        parent_id: None,
        title: Some("src/gateway.ts".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("src/gateway.ts".to_string()),
        source_url: "https://github.com/example/repo/blob/main/src/gateway.ts".to_string(),
        line_range: Some(LineRange::new(1, 25)),
        byte_range: None,
        language: Some("typescript".to_string()),
        content_hash: CdmSection::compute_hash(ts_code),
        body: ts_code.to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(100)
        .with_max_tokens(200);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_code_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("chunking succeeds");

    assert!(!chunks.is_empty());
    let all_symbols: Vec<_> = chunks.iter().flat_map(|c| c.ast_symbols.iter()).collect();

    let names: Vec<_> = all_symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"RouteHandlerConfig") || names.contains(&"ApiGateway"));
}

#[test]
fn test_oversized_function_clean_splitting_with_signature_header() {
    // Generate a function that exceeds the max token threshold
    let mut big_function =
        String::from("pub fn compute_huge_matrix_calculation(input: &[f64]) -> Vec<f64> {\n");
    for i in 0..100 {
        big_function.push_str(&format!(
            "    let step_{i} = input.iter().map(|&x| x * {i}.5).sum::<f64>();\n"
        ));
    }
    big_function.push_str("    vec![1.0, 2.0, 3.0]\n}\n");

    let section = CdmSection {
        section_id: "sec_big_1".to_string(),
        parent_id: None,
        title: Some("src/matrix.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("src/matrix.rs".to_string()),
        source_url: "https://github.com/example/repo/blob/main/src/matrix.rs".to_string(),
        line_range: Some(LineRange::new(1, 103)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: CdmSection::compute_hash(&big_function),
        body: big_function,
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(100)
        .with_max_tokens(150);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_code_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("chunking succeeds");

    assert!(
        chunks.len() > 1,
        "oversized function must be split across multiple chunks"
    );
    assert!(report.oversized_splits_count >= 1);

    // Verify context header and signature retention on splits
    for (idx, chunk) in chunks.iter().enumerate() {
        assert!(
            chunk.content.contains("compute_huge_matrix_calculation"),
            "every split chunk must retain the function name/signature in header or body"
        );
        let part_expected = format!("Part {}", idx + 1);
        assert!(chunk.title.as_ref().unwrap().contains(&part_expected));
    }
}

#[test]
fn test_go_ast_chunking_structs_and_methods() {
    let go_code = r#"package server

import (
    "fmt"
    "net/http"
)

type ServerConfig struct {
    Host string
    Port int
}

func NewServerConfig(host string, port int) *ServerConfig {
    return &ServerConfig{
        Host: host,
        Port: port,
    }
}

func (s *ServerConfig) ListenAndServe() error {
    addr := fmt.Sprintf("%s:%d", s.Host, s.Port)
    return http.ListenAndServe(addr, nil)
}
"#;

    let section = CdmSection {
        section_id: "sec_go_1".to_string(),
        parent_id: None,
        title: Some("server.go".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("server.go".to_string()),
        source_url: "https://example.com/server.go".to_string(),
        line_range: Some(LineRange::new(1, 22)),
        byte_range: None,
        language: Some("go".to_string()),
        content_hash: CdmSection::compute_hash(go_code),
        body: go_code.to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default().with_target_tokens(80);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_code_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("go chunking succeeds");

    assert!(!chunks.is_empty());
    let all_symbols: Vec<_> = chunks.iter().flat_map(|c| c.ast_symbols.iter()).collect();

    let names: Vec<_> = all_symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"ServerConfig")
            || names.contains(&"NewServerConfig")
            || names.contains(&"ListenAndServe")
    );
}

#[test]
fn test_generic_code_fallback_when_no_symbols() {
    let bash_script = r#"#!/usr/bin/env bash
set -euo pipefail

echo "Deploying application..."
export NODE_ENV=production
npm run build
pm2 reload all
echo "Deployment successful."
"#;

    let section = CdmSection {
        section_id: "sec_sh_1".to_string(),
        parent_id: None,
        title: Some("deploy.sh".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("deploy.sh".to_string()),
        source_url: "https://example.com/deploy.sh".to_string(),
        line_range: None,
        byte_range: None,
        language: Some("shell".to_string()),
        content_hash: CdmSection::compute_hash(bash_script),
        body: bash_script.to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default();
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_code_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("shell script chunking succeeds");

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].kind, ChunkKind::CodeBlock);
    assert!(chunks[0].content.contains("Deploying application"));
}
