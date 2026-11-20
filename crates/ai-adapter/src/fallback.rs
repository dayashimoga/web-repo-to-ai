//! Zero-key fallback engine providing deterministic CPU-only operations.
//!
//! Enforces Section 3.4 of the GLOBAL_ENGINEERING_CONTRACT:
//! "Zero Mandatory External AI / Paid Dependencies: All core document extraction,
//! normalization, deduplication, chunking, and export logic execute locally on CPU
//! without AI assistance. AI provider adapters serve solely as optional, opt-in
//! post-processing extensions. Core extraction MUST ALWAYS succeed without AI keys."

use crate::models::{
    ChatChoice, ChatCompletionRequest, ChatCompletionResponse, ChatFinishReason, ChatMessage,
    ChatResponseFormat, DocumentSummarizationRequest, DocumentSummarizationResponse,
    EmbeddingRequest, EmbeddingResponse, SectionEnrichmentRequest, SectionEnrichmentResponse,
    SummaryFocus, TokenUsage,
};
use chrono::Utc;
use std::collections::HashSet;

/// Deterministic, CPU-only engine providing high-fidelity fallback operations when external AI
/// providers are disabled, unconfigured, or offline.
#[derive(Debug, Clone, Default)]
pub struct ZeroKeyFallbackEngine;

impl ZeroKeyFallbackEngine {
    /// Construct a new zero-key fallback engine.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Generate an extractive document summary on the local CPU without external LLM calls.
    #[must_use]
    pub fn fallback_summarize(
        &self,
        request: &DocumentSummarizationRequest,
    ) -> DocumentSummarizationResponse {
        let content = &request.content;
        let mut detected_title: Option<String> = None;
        let mut headings: Vec<String> = Vec::new();
        let mut paragraphs: Vec<String> = Vec::new();

        // 1. Parse markdown structure and headings
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("# ") && detected_title.is_none() {
                detected_title = Some(trimmed.trim_start_matches("# ").trim().to_string());
            } else if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
                let h = trimmed.trim_start_matches('#').trim().to_string();
                if !headings.contains(&h) {
                    headings.push(h);
                }
            } else if !trimmed.starts_with("```")
                && !trimmed.starts_with('-')
                && !trimmed.starts_with('*')
            {
                paragraphs.push(trimmed.to_string());
            }
        }

        let title = if !request.title.is_empty() {
            request.title.clone()
        } else if let Some(dt) = detected_title {
            dt
        } else {
            "Document Overview".to_string()
        };

        // 2. Select salient sentences for extractive summary
        let mut summary_sentences: Vec<String> = Vec::new();
        summary_sentences.push(format!("Document: {title}."));

        for p in &paragraphs {
            if summary_sentences.len() >= 4 {
                break;
            }
            if let Some(sentence) = extract_first_sentence(p) {
                if !summary_sentences.iter().any(|s| s.contains(&sentence)) {
                    summary_sentences.push(sentence);
                }
            }
        }

        if summary_sentences.len() == 1 {
            summary_sentences.push("Extracted content overview.".to_string());
        }

        let summary_text = summary_sentences.join(" ");

        // 3. Extract topics according to focus
        let mut key_topics: Vec<String> = match request.focus {
            SummaryFocus::Architecture => {
                let mut base = vec!["architecture".to_string(), "system-design".to_string()];
                base.extend(headings.iter().take(3).cloned());
                base
            }
            SummaryFocus::Security => {
                let mut base = vec!["security".to_string(), "access-control".to_string()];
                base.extend(headings.iter().take(3).cloned());
                base
            }
            SummaryFocus::ApiOverview => {
                let mut base = vec!["api".to_string(), "interfaces".to_string()];
                base.extend(headings.iter().take(3).cloned());
                base
            }
            SummaryFocus::KeyInsights | SummaryFocus::General => {
                headings.iter().take(5).cloned().collect()
            }
        };

        if key_topics.is_empty() {
            key_topics.push("general".to_string());
        }

        let est_prompt_tokens = content.split_whitespace().count() * 4 / 3;
        let est_completion_tokens = summary_text.split_whitespace().count() * 4 / 3;

        DocumentSummarizationResponse {
            summary: summary_text,
            key_topics,
            token_usage: TokenUsage::new(est_prompt_tokens, est_completion_tokens),
            model: "extractive-cpu-fallback".to_string(),
            latency_ms: 0,
        }
    }

    /// Generate heuristic section enrichment on the local CPU.
    #[must_use]
    pub fn fallback_enrich_section(
        &self,
        request: &SectionEnrichmentRequest,
    ) -> SectionEnrichmentResponse {
        let content = &request.content;
        let heading = request
            .section_title
            .as_deref()
            .unwrap_or(&request.section_kind);

        // 1. Generate extractive summary
        let mut summary_lines: Vec<String> = Vec::new();
        if !heading.is_empty() {
            summary_lines.push(format!("Section '{heading}':"));
        }

        for line in content.lines() {
            let t = line.trim();
            if !t.is_empty() && !t.starts_with('#') && !t.starts_with("```") {
                if let Some(s) = extract_first_sentence(t) {
                    summary_lines.push(s);
                    break;
                }
            }
        }

        if summary_lines.len() <= 1 {
            summary_lines.push("Contains technical declarations and documentation.".to_string());
        }

        // 2. Synthesize tags
        let mut tags: HashSet<String> = HashSet::new();
        if !heading.is_empty() {
            let h_clean = heading.to_ascii_lowercase().replace(' ', "-");
            tags.insert(h_clean);
        }

        if let Some(ref lang) = request.language {
            tags.insert(lang.to_ascii_lowercase());
        }

        // Detect code keywords
        if content.contains("fn ") || content.contains("pub fn ") {
            tags.insert("function".to_string());
        }
        if content.contains("struct ") || content.contains("class ") {
            tags.insert("data-structure".to_string());
        }
        if content.contains("enum ") {
            tags.insert("enum".to_string());
        }
        if content.contains("impl ") || content.contains("interface ") {
            tags.insert("interface".to_string());
        }

        let est_prompt_tokens = content.split_whitespace().count() * 4 / 3;
        let est_completion_tokens = 25;

        SectionEnrichmentResponse {
            suggested_title: format!("{heading} (Processed)"),
            concise_description: summary_lines.join(" "),
            tags: tags.into_iter().collect(),
            token_usage: TokenUsage::new(est_prompt_tokens, est_completion_tokens),
            model: "heuristic-cpu-fallback".to_string(),
            latency_ms: 0,
        }
    }

    /// Generate deterministic Blake3 $L_2$ unit-normalized embeddings on the CPU.
    #[must_use]
    pub fn fallback_embeddings(&self, request: &EmbeddingRequest) -> EmbeddingResponse {
        let dim = request.dimensions.unwrap_or(384).max(1);
        let mut embeddings = Vec::with_capacity(request.inputs.len());

        for text in &request.inputs {
            let embedding = generate_deterministic_embedding(text, dim);
            embeddings.push(embedding);
        }

        let total_tokens = request
            .inputs
            .iter()
            .map(|t| t.split_whitespace().count() * 4 / 3)
            .sum();

        EmbeddingResponse {
            model: request.model.clone(),
            embeddings,
            usage: TokenUsage::new(total_tokens, 0),
            dimensions: dim,
            latency_ms: 0,
        }
    }

    /// Generate a deterministic chat response on the CPU.
    #[must_use]
    pub fn fallback_chat(&self, request: &ChatCompletionRequest) -> ChatCompletionResponse {
        let is_json = request.response_format == Some(ChatResponseFormat::JsonObject);

        let content = if is_json {
            serde_json::json!({
                "status": "zero_key_fallback",
                "mode": "offline_cpu",
                "message": "AI features executed via deterministic CPU fallback."
            })
            .to_string()
        } else {
            "Zero-key fallback: local CPU pipeline executed without external AI.".to_string()
        };

        ChatCompletionResponse {
            id: format!("fallback-{}", uuid::Uuid::new_v4()),
            model: "deterministic-cpu-fallback".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage::assistant(content),
                finish_reason: ChatFinishReason::Stop,
            }],
            usage: TokenUsage::new(10, 10),
            created_at: Utc::now(),
            latency_ms: 0,
        }
    }
}

/// Helper extracting the first complete sentence from a line of text.
fn extract_first_sentence(text: &str) -> Option<String> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }

    for (i, c) in t.char_indices() {
        if (c == '.' || c == '!' || c == '?') && i >= 10 {
            return Some(t[..=i].to_string());
        }
    }

    if t.len() > 120 {
        Some(format!("{}...", &t[..117]))
    } else {
        Some(t.to_string())
    }
}

/// Deterministic Blake3 pseudo-embedding normalized to unit length ($L_2 = 1.0$).
fn generate_deterministic_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    let mut vec = Vec::with_capacity(dimensions);
    let mut round = 0u32;

    while vec.len() < dimensions {
        let mut hasher = blake3::Hasher::new();
        hasher.update(text.as_bytes());
        hasher.update(&round.to_le_bytes());
        let hash = hasher.finalize();

        for chunk in hash.as_bytes().chunks_exact(4) {
            if vec.len() >= dimensions {
                break;
            }
            let val = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let norm_val = (val as f64 / u32::MAX as f64) * 2.0 - 1.0;
            vec.push(norm_val as f32);
        }
        round += 1;
    }

    // Compute L2 norm and normalize
    let sum_sq: f32 = vec.iter().map(|v| v * v).sum();
    let norm = sum_sq.sqrt();
    if norm > 1e-9 {
        for v in &mut vec {
            *v /= norm;
        }
    }

    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_key_extractive_summarize() {
        let engine = ZeroKeyFallbackEngine::new();
        let doc = r#"# Architecture Overview
This document outlines the distributed pipeline architecture.
The system uses microservices with asynchronous event messaging.

## Ingestion Layer
The ingestion layer fetches and parses incoming web repositories.
All network endpoints are strictly validated for SSRF defenses.

## Storage Engine
Data is stored durably in PostgreSQL with blob storage.
"#;
        let req = DocumentSummarizationRequest::new("doc-1", "Architecture Overview", doc)
            .with_focus(SummaryFocus::Architecture);
        let resp = engine.fallback_summarize(&req);

        assert_eq!(resp.model, "extractive-cpu-fallback");
        assert!(resp.summary.contains("Architecture Overview"));
        assert!(resp.key_topics.contains(&"architecture".to_string()));
    }

    #[test]
    fn test_zero_key_fallback_embeddings_unit_norm() {
        let engine = ZeroKeyFallbackEngine::new();
        let req = EmbeddingRequest::new("test-model", vec!["hello world".to_string()])
            .with_dimensions(384);
        let resp = engine.fallback_embeddings(&req);

        assert_eq!(resp.embeddings.len(), 1);
        let emb = &resp.embeddings[0];
        assert_eq!(emb.len(), 384);

        let l2_norm: f32 = emb.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((l2_norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_zero_key_fallback_chat_json() {
        let engine = ZeroKeyFallbackEngine::new();
        let req =
            ChatCompletionRequest::new("model", vec![ChatMessage::user("ping")]).with_json_format();
        let resp = engine.fallback_chat(&req);

        let parsed: serde_json::Value =
            serde_json::from_str(&resp.choices[0].message.content).unwrap();
        assert_eq!(parsed["status"], "zero_key_fallback");
    }
}
