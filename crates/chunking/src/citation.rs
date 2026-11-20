//! # Citation Generator & Source Provenance Anchoring
//!
//! Generates authoritative source citations, GitHub permalinks with commit hashes and line spans,
//! web documentation anchors, PDF page numbers, and structured specification pointers.

use web_repo_to_ai_cdm::{CdmDocument, CdmSection, LineRange, SourceType};

use crate::model::{ChunkCitation, ChunkId, ChunkSymbolMetadata};

/// Generates an authoritative citation anchoring a chunk to its original source.
#[must_use]
pub fn generate_chunk_citation(
    doc: Option<&CdmDocument>,
    section: &CdmSection,
    chunk_id: &ChunkId,
    line_range: &LineRange,
    symbol: Option<&ChunkSymbolMetadata>,
) -> ChunkCitation {
    let citation_id = format!("cit_{}", chunk_id.as_str());
    let file_path = section
        .original_path
        .clone()
        .or_else(|| section.attributes.get("path").cloned());

    let start_line = line_range.start_line as usize;
    let end_line = line_range.end_line as usize;
    let line_span = Some((start_line, end_line));

    // Determine base URL
    let base_url = if !section.source_url.is_empty() {
        section.source_url.clone()
    } else if let Some(d) = doc {
        d.source_identity.canonical_url.clone()
    } else if let Some(ref p) = file_path {
        format!("file:///{p}")
    } else {
        "file:///document".to_string()
    };

    // Check if Git / GitHub source
    let is_github = doc
        .map(|d| {
            d.source_type == SourceType::GitHubPublic
                || d.source_type == SourceType::GitHubPrivate
                || d.source_identity.domain.as_deref() == Some("github.com")
                || d.source_identity.canonical_url.contains("github.com/")
        })
        .unwrap_or(false)
        || base_url.contains("github.com/");

    if is_github {
        return generate_github_citation(
            doc,
            section,
            citation_id,
            file_path,
            start_line,
            end_line,
            &base_url,
            symbol,
        );
    }

    // Check if PDF source
    let is_pdf = doc
        .map(|d| d.source_type == SourceType::DocumentPdf)
        .unwrap_or(false)
        || section.attributes.contains_key("page_number")
        || section.attributes.contains_key("ocr_page_number");

    if is_pdf {
        return generate_pdf_citation(doc, section, citation_id, file_path, line_span, &base_url);
    }

    // Check if Structured Spec source (OpenAPI, JSON Schema, Notebook)
    let is_structured = doc
        .map(|d| d.source_type == SourceType::StructuredDocument)
        .unwrap_or(false)
        || section.attributes.contains_key("operation_path")
        || section.attributes.contains_key("cell_index");

    if is_structured {
        return generate_structured_citation(
            doc,
            section,
            citation_id,
            file_path,
            line_span,
            &base_url,
        );
    }

    // Check if Web Article or Documentation
    let is_web = doc
        .map(|d| {
            d.source_type == SourceType::WebpageStatic
                || d.source_type == SourceType::WebpageDynamic
        })
        .unwrap_or(false)
        || base_url.starts_with("http://")
        || base_url.starts_with("https://");

    if is_web {
        return generate_web_citation(
            doc,
            section,
            citation_id,
            file_path,
            line_span,
            &base_url,
            symbol,
        );
    }

    // Fallback local citation
    generate_local_citation(
        section,
        citation_id,
        file_path,
        start_line,
        end_line,
        &base_url,
    )
}

#[allow(clippy::too_many_arguments)]
fn generate_github_citation(
    doc: Option<&CdmDocument>,
    section: &CdmSection,
    citation_id: String,
    file_path: Option<String>,
    start_line: usize,
    end_line: usize,
    base_url: &str,
    symbol: Option<&ChunkSymbolMetadata>,
) -> ChunkCitation {
    let repo_coords = doc
        .and_then(|d| {
            if let (Some(o), Some(r)) =
                (&d.source_identity.repo_owner, &d.source_identity.repo_name)
            {
                Some((o.clone(), r.clone()))
            } else {
                parse_github_coords(&d.source_identity.canonical_url)
            }
        })
        .or_else(|| parse_github_coords(base_url));

    let commit_sha = doc
        .and_then(|d| d.revision.git_commit.clone())
        .or_else(|| doc.and_then(|d| d.source_identity.git_ref.clone()))
        .unwrap_or_else(|| "main".to_string());

    let clean_path = file_path
        .as_deref()
        .unwrap_or("file")
        .trim_start_matches('/')
        .trim_start_matches("./");

    let anchor = if start_line == end_line {
        format!("L{start_line}")
    } else {
        format!("L{start_line}-L{end_line}")
    };

    let permalink = if let Some((owner, repo)) = repo_coords {
        format!("https://github.com/{owner}/{repo}/blob/{commit_sha}/{clean_path}#{anchor}")
    } else if base_url.contains("/blob/") {
        let clean_base = base_url.split('#').next().unwrap_or(base_url);
        format!("{clean_base}#{anchor}")
    } else {
        format!("{base_url}#{anchor}")
    };

    let short_sha = if commit_sha.len() >= 7 {
        &commit_sha[..7]
    } else {
        &commit_sha
    };

    let display_citation = if let Some(sym) = symbol {
        format!(
            "{clean_path}:{start_line}-{end_line} > {} ({short_sha})",
            sym.name
        )
    } else {
        format!("{clean_path}:{start_line}-{end_line} ({short_sha})")
    };

    let markdown_citation = format!("[{clean_path}#{anchor}]({permalink})");

    ChunkCitation {
        citation_id,
        source_url: base_url.to_string(),
        permalink,
        display_citation,
        markdown_citation,
        file_path: Some(clean_path.to_string()),
        line_span: Some((start_line, end_line)),
        commit_sha: Some(commit_sha),
        source_title: section.title.clone(),
        anchor: Some(anchor),
    }
}

fn generate_web_citation(
    doc: Option<&CdmDocument>,
    section: &CdmSection,
    citation_id: String,
    file_path: Option<String>,
    line_span: Option<(usize, usize)>,
    base_url: &str,
    symbol: Option<&ChunkSymbolMetadata>,
) -> ChunkCitation {
    let clean_base = base_url.split('#').next().unwrap_or(base_url);
    let anchor = section
        .attributes
        .get("anchor")
        .or_else(|| section.attributes.get("heading_slug"))
        .or_else(|| section.attributes.get("slug"))
        .cloned()
        .or_else(|| symbol.map(|s| format!("symbol-{}", s.name)));

    let permalink = if let Some(ref a) = anchor {
        format!("{clean_base}#{a}")
    } else {
        clean_base.to_string()
    };

    let doc_title = doc
        .and_then(|d| d.metadata.title.clone())
        .or_else(|| section.title.clone())
        .unwrap_or_else(|| clean_base.to_string());

    let display_citation = if let Some(ref sec_title) = section.title {
        if sec_title != &doc_title {
            format!("{doc_title} > {sec_title}")
        } else {
            doc_title.clone()
        }
    } else if let Some(sym) = symbol {
        format!("{doc_title} > {}", sym.name)
    } else {
        doc_title.clone()
    };

    let markdown_citation = format!("[{display_citation}]({permalink})");

    ChunkCitation {
        citation_id,
        source_url: clean_base.to_string(),
        permalink,
        display_citation,
        markdown_citation,
        file_path,
        line_span,
        commit_sha: None,
        source_title: Some(doc_title),
        anchor,
    }
}

fn generate_pdf_citation(
    doc: Option<&CdmDocument>,
    section: &CdmSection,
    citation_id: String,
    file_path: Option<String>,
    line_span: Option<(usize, usize)>,
    base_url: &str,
) -> ChunkCitation {
    let page_number = section
        .attributes
        .get("page_number")
        .or_else(|| section.attributes.get("ocr_page_number"))
        .cloned()
        .unwrap_or_else(|| "1".to_string());

    let anchor = format!("page={page_number}");
    let clean_base = base_url.split('#').next().unwrap_or(base_url);
    let permalink = format!("{clean_base}#{anchor}");

    let doc_title = doc
        .and_then(|d| d.metadata.title.clone())
        .or_else(|| section.title.clone())
        .or_else(|| file_path.clone())
        .unwrap_or_else(|| "Document.pdf".to_string());

    let display_citation = format!("{doc_title} (p. {page_number})");
    let markdown_citation = format!("[{doc_title} (p. {page_number})]({permalink})");

    ChunkCitation {
        citation_id,
        source_url: clean_base.to_string(),
        permalink,
        display_citation,
        markdown_citation,
        file_path,
        line_span,
        commit_sha: None,
        source_title: Some(doc_title),
        anchor: Some(anchor),
    }
}

fn generate_structured_citation(
    doc: Option<&CdmDocument>,
    section: &CdmSection,
    citation_id: String,
    file_path: Option<String>,
    line_span: Option<(usize, usize)>,
    base_url: &str,
) -> ChunkCitation {
    let path_label = file_path
        .clone()
        .or_else(|| section.title.clone())
        .unwrap_or_else(|| "specification".to_string());

    let (anchor, display_citation) = if let Some(op) = section.attributes.get("operation_path") {
        let anchor = op
            .replace([' ', '/'], "-")
            .trim_start_matches('-')
            .to_string();
        let display = format!("{path_label} > {op}");
        (Some(anchor), display)
    } else if let Some(cell) = section.attributes.get("cell_index") {
        let anchor = format!("cell-{cell}");
        let display = format!("{path_label} > Cell {cell}");
        (Some(anchor), display)
    } else if let Some(schema) = section.attributes.get("schema_name") {
        let anchor = format!("schema-{schema}");
        let display = format!("{path_label} > {schema}");
        (Some(anchor), display)
    } else {
        (None, path_label.clone())
    };

    let clean_base = base_url.split('#').next().unwrap_or(base_url);
    let permalink = if let Some(ref a) = anchor {
        format!("{clean_base}#{a}")
    } else {
        clean_base.to_string()
    };

    let markdown_citation = format!("[{display_citation}]({permalink})");

    ChunkCitation {
        citation_id,
        source_url: clean_base.to_string(),
        permalink,
        display_citation,
        markdown_citation,
        file_path,
        line_span,
        commit_sha: None,
        source_title: doc.and_then(|d| d.metadata.title.clone()),
        anchor,
    }
}

fn generate_local_citation(
    section: &CdmSection,
    citation_id: String,
    file_path: Option<String>,
    start_line: usize,
    end_line: usize,
    base_url: &str,
) -> ChunkCitation {
    let clean_path = file_path
        .as_deref()
        .unwrap_or("document")
        .trim_start_matches('/')
        .trim_start_matches("./");

    let anchor = if start_line == end_line {
        format!("L{start_line}")
    } else {
        format!("L{start_line}-L{end_line}")
    };

    let permalink = if base_url.starts_with("file:///") {
        format!("{base_url}#{anchor}")
    } else {
        format!("file:///{clean_path}#{anchor}")
    };

    let display_citation = format!("{clean_path}:{start_line}-{end_line}");
    let markdown_citation = format!("[{clean_path}#{anchor}]({permalink})");

    ChunkCitation {
        citation_id,
        source_url: base_url.to_string(),
        permalink,
        display_citation,
        markdown_citation,
        file_path: Some(clean_path.to_string()),
        line_span: Some((start_line, end_line)),
        commit_sha: None,
        source_title: section.title.clone(),
        anchor: Some(anchor),
    }
}

/// Helper to parse GitHub owner and repository name from a URL.
#[must_use]
pub fn parse_github_coords(url: &str) -> Option<(String, String)> {
    let lower = url.to_lowercase();
    let idx = lower.find("github.com/")?;
    let parts: Vec<&str> = lower[idx + 11..].split('/').collect();

    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        let owner = parts[0].to_string();
        let repo = parts[1].trim_end_matches(".git").to_string();
        Some((owner, repo))
    } else {
        None
    }
}
