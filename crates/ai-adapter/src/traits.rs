//! Core asynchronous trait definitions for provider-independent AI adapters.

use crate::error::AiAdapterError;
use crate::models::{
    AiHealthStatus, AiModelDescriptor, AiProviderType, ChatCompletionRequest,
    ChatCompletionResponse, ChatMessage, DocumentSummarizationRequest,
    DocumentSummarizationResponse, EmbeddingRequest, EmbeddingResponse, SectionEnrichmentRequest,
    SectionEnrichmentResponse, SummaryFocus,
};
use async_trait::async_trait;

/// Provider-independent abstract interface for AI assistance capabilities.
///
/// Implemented by local backends (e.g. Ollama), external API adapters,
/// and test mocks. Core extraction and normalization logic interact solely
/// with this trait, ensuring zero hard vendor lock-in.
#[async_trait]
pub trait AiAdapter: Send + Sync + std::fmt::Debug {
    /// Return the provider category or backend identity.
    fn provider_type(&self) -> AiProviderType;

    /// Whether this adapter is configured and active.
    ///
    /// When `false`, the platform operates in zero-key fallback mode where
    /// all core document operations proceed without AI calls.
    fn is_enabled(&self) -> bool;

    /// Probe adapter connectivity and health.
    async fn health_check(&self) -> Result<AiHealthStatus, AiAdapterError>;

    /// List all available model descriptors supported by this adapter.
    async fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiAdapterError>;

    /// Execute a chat completion prompt.
    async fn complete_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiAdapterError>;

    /// Generate dense vector embeddings for one or more text inputs.
    async fn generate_embeddings(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiAdapterError>;

    /// High-level document summarization helper with structured default prompt engineering.
    async fn summarize_document(
        &self,
        request: DocumentSummarizationRequest,
    ) -> Result<DocumentSummarizationResponse, AiAdapterError> {
        let focus_instruction = match request.focus {
            SummaryFocus::General => {
                "Provide a comprehensive, high-density technical summary of the document's content, purpose, and key components."
            }
            SummaryFocus::Architecture => {
                "Focus on system architecture, crate/package decomposition, dependency flow, data structures, and lifecycle patterns."
            }
            SummaryFocus::Security => {
                "Focus on security postures, authentication/authorization models, data boundaries, threat vectors, and cryptography."
            }
            SummaryFocus::ApiOverview => {
                "Focus on public REST/RPC API endpoints, request/response models, error handling, and client integration protocols."
            }
            SummaryFocus::KeyInsights => {
                "Highlight the top actionable insights, critical decisions, algorithmic novelties, and quantitative metrics."
            }
        };

        let system_msg = ChatMessage::system(format!(
            "You are an expert AI documentation analyst. Your task is to analyze documents converted for LLM context.\n\
             {}\n\
             Guidelines:\n\
             - Output clear, factual Markdown.\n\
             - Avoid fluff or conversational preambles.\n\
             - End your response with a line formatted as: 'Key Topics: topic1, topic2, topic3'",
            focus_instruction
        ));

        let user_msg = ChatMessage::user(format!(
            "# Document: {}\n\n{}",
            request.title, request.content
        ));

        let model = request
            .model
            .clone()
            .unwrap_or_else(|| "default-chat".to_string());

        let mut chat_req = ChatCompletionRequest::new(model, vec![system_msg, user_msg]);
        if let Some(max_tok) = request.max_summary_tokens {
            chat_req = chat_req.with_max_tokens(max_tok);
        }

        let resp = self.complete_chat(chat_req).await?;
        let text = resp.primary_text().unwrap_or("").trim();

        // Extract key topics if present in trailing line
        let mut key_topics = Vec::new();
        let mut summary_lines = Vec::new();

        for line in text.lines() {
            if let Some(topics_str) = line.strip_prefix("Key Topics:") {
                for topic in topics_str.split(',') {
                    let cleaned = topic.trim().trim_matches('"').trim();
                    if !cleaned.is_empty() {
                        key_topics.push(cleaned.to_string());
                    }
                }
            } else {
                summary_lines.push(line);
            }
        }

        let summary = summary_lines.join("\n").trim().to_string();

        Ok(DocumentSummarizationResponse {
            summary,
            key_topics,
            token_usage: resp.usage,
            model: resp.model,
            latency_ms: resp.latency_ms,
        })
    }

    /// High-level section metadata enrichment helper with structured prompt engineering.
    async fn enrich_section(
        &self,
        request: SectionEnrichmentRequest,
    ) -> Result<SectionEnrichmentResponse, AiAdapterError> {
        let system_msg = ChatMessage::system(
            "You are an expert code and documentation analyzer. Enrich this section with metadata.\n\
             Your response must follow this exact format:\n\
             Title: <suggested concise title>\n\
             Description: <1-2 sentence description of purpose>\n\
             Tags: <comma-separated keywords>",
        );

        let user_content = format!(
            "Section Kind: {}\nFile Path: {}\nLanguage: {}\nCurrent Title: {}\n\nContent:\n{}",
            request.section_kind,
            request.file_path.as_deref().unwrap_or("None"),
            request.language.as_deref().unwrap_or("Text"),
            request.section_title.as_deref().unwrap_or("None"),
            request.content
        );

        let user_msg = ChatMessage::user(user_content);
        let model = request
            .model
            .clone()
            .unwrap_or_else(|| "default-chat".to_string());

        let chat_req = ChatCompletionRequest::new(model, vec![system_msg, user_msg]);
        let resp = self.complete_chat(chat_req).await?;
        let text = resp.primary_text().unwrap_or("").trim();

        let mut suggested_title = request.section_title.clone().unwrap_or_default();
        let mut concise_description = String::new();
        let mut tags = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(val) = trimmed.strip_prefix("Title:") {
                let t = val.trim();
                if !t.is_empty() {
                    suggested_title = t.to_string();
                }
            } else if let Some(val) = trimmed.strip_prefix("Description:") {
                concise_description = val.trim().to_string();
            } else if let Some(val) = trimmed.strip_prefix("Tags:") {
                for tag in val.split(',') {
                    let cleaned = tag.trim().trim_matches('"').trim();
                    if !cleaned.is_empty() {
                        tags.push(cleaned.to_string());
                    }
                }
            }
        }

        if suggested_title.is_empty() {
            suggested_title = format!("{} Section", request.section_kind);
        }
        if concise_description.is_empty() {
            concise_description = format!(
                "Content block of kind {} with {} bytes.",
                request.section_kind,
                request.content.len()
            );
        }

        Ok(SectionEnrichmentResponse {
            suggested_title,
            concise_description,
            tags,
            token_usage: resp.usage,
            model: resp.model,
            latency_ms: resp.latency_ms,
        })
    }
}
