//! # Tokenizer Adapter Architecture Integration Tests (`crates/tokenizer`)
//!
//! Validates exact BPE tokenization, calibrated heuristic estimation with explicit uncertainty bounds,
//! multi-provider adapter resolution, token-budget truncation, document-level enrichment, and
//! telemetry serialization.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    LineRange, RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::{
    count_section_tokens, enrich_document_token_metrics, CalibratedHeuristicAdapter, ClaudeAdapter,
    DocumentTokenSummary, GeminiAdapter, LlamaAdapter, ModelFamily, OpenAiAdapter, TargetModel,
    TokenPrecision, TokenizerAdapter, TokenizerRegistry,
};

fn make_test_section(id: &str, path: &str, body: &str) -> CdmSection {
    CdmSection {
        section_id: id.to_string(),
        parent_id: None,
        title: Some(id.to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some(path.to_string()),
        source_url: format!("https://example.com/{path}"),
        line_range: Some(LineRange::new(1, 10)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: CdmSection::compute_hash(body),
        body: body.to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    }
}

fn build_sample_cdm_document() -> CdmDocument {
    let s1 = make_test_section(
        "sec-dispatcher",
        "src/dispatcher.rs",
        "pub struct Dispatcher { pub workers: usize }\npub fn run() {}\n",
    );
    let s2 = make_test_section(
        "sec-worker",
        "src/worker.rs",
        "pub async fn worker_loop() -> Result<(), Error> {\n    println!(\"Loop running\");\n    Ok(())\n}\n",
    );
    let s3 = make_test_section(
        "sec-config",
        "config.json",
        "{\n  \"max_concurrency\": 16,\n  \"timeout_seconds\": 30\n}\n",
    );

    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new(
            "https://example.com/repo",
            "https://example.com/repo",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://example.com/repo".to_string(),
        retrieved_at: Utc::now(),
        revision: RevisionInfo::default(),
        content_type: "text/markdown".to_string(),
        metadata: DocumentMetadata::default(),
        hierarchy: HierarchyNode::root("root"),
        sections: vec![s1, s2, s3],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    }
}

#[test]
fn test_openai_adapter_exact_bpe_token_counting() {
    let adapter_o200k = OpenAiAdapter::for_model(TargetModel::Gpt4o);
    assert_eq!(adapter_o200k.name(), "o200k_base");
    assert_eq!(adapter_o200k.model_family(), ModelFamily::OpenAi);
    assert_eq!(adapter_o200k.target_model(), TargetModel::Gpt4o);
    assert!(adapter_o200k.is_exact());

    let text = "The distributed engine coordinates worker tasks across 128 isolated containers.";
    let res = adapter_o200k.count_tokens(text);

    assert_eq!(res.precision, TokenPrecision::Exact);
    assert!(res.precision.is_exact());
    assert_eq!(res.precision.uncertainty_percentage(), 0.0);
    assert_eq!(res.tokenizer_name, "o200k_base");
    assert!(res.count >= 10);
    assert!(res.count <= 20);
    assert!(res.chars_per_token > 3.0);
    assert!(res.confidence_range.is_none());

    let adapter_cl100k = OpenAiAdapter::for_model(TargetModel::Gpt4Turbo);
    assert_eq!(adapter_cl100k.name(), "cl100k_base");
    let res_cl100k = adapter_cl100k.count_tokens(text);
    assert_eq!(res_cl100k.precision, TokenPrecision::Exact);
}

#[test]
fn test_claude_gemini_and_llama_adapters() {
    let claude = ClaudeAdapter::for_model(TargetModel::Claude35Sonnet);
    assert_eq!(claude.name(), "claude_bpe");
    assert_eq!(claude.model_family(), ModelFamily::Anthropic);
    assert_eq!(claude.context_window_limit(), 200_000);

    let text = "Claude 3.5 Sonnet processing high-assurance code review pipelines.";
    let res_claude = claude.count_tokens(text);
    assert!(res_claude.precision.is_exact());
    assert!(res_claude.count > 0);

    let gemini = GeminiAdapter::for_model(TargetModel::Gemini15Pro);
    assert_eq!(gemini.name(), "gemini_sp");
    assert_eq!(gemini.model_family(), ModelFamily::Google);
    assert_eq!(gemini.context_window_limit(), 2_000_000);

    let res_gemini = gemini.count_tokens(text);
    assert!(res_gemini.precision.is_exact());
    assert!(res_gemini.count > 0);

    let llama = LlamaAdapter::for_model(TargetModel::Llama3);
    assert_eq!(llama.name(), "llama3_bpe");
    assert_eq!(llama.model_family(), ModelFamily::Meta);
    assert_eq!(llama.context_window_limit(), 128_000);

    let res_llama = llama.count_tokens(text);
    assert!(res_llama.precision.is_exact());
    assert!(res_llama.count > 0);
}

#[test]
fn test_calibrated_heuristic_adapter_and_uncertainty() {
    let adapter = CalibratedHeuristicAdapter::default();
    assert_eq!(adapter.name(), "calibrated_heuristic");
    assert!(!adapter.is_exact());

    let text = "Reliable asynchronous messaging queue with exponential backoff retries.";
    let res = adapter.count_tokens(text);

    assert_eq!(
        res.precision,
        TokenPrecision::CalibratedEstimate {
            uncertainty_pct: 7.5
        }
    );
    assert!(!res.precision.is_exact());
    assert_eq!(res.precision.uncertainty_percentage(), 7.5);
    assert!(res.confidence_range.is_some());

    let (min_bound, max_bound) = res.confidence_range.unwrap();
    assert!(min_bound <= res.count);
    assert!(max_bound >= res.count);
}

#[test]
fn test_token_truncation_exact_boundaries() {
    let adapter = OpenAiAdapter::for_model(TargetModel::Gpt4o);
    let sample = "Sentence one. Sentence two. Sentence three. Sentence four. Sentence five.";

    let max_tokens = 6;
    let res = adapter.truncate_tokens(sample, max_tokens);

    assert!(res.was_truncated);
    assert!(res.resulting_tokens <= max_tokens);
    assert!(res.omitted_tokens > 0);
    assert!(sample.starts_with(&res.truncated_text));
    assert!(res
        .truncated_text
        .is_char_boundary(res.truncated_text.len()));
}

#[test]
fn test_tokenizer_registry_resolution_and_defaults() {
    let registry = TokenizerRegistry::default();

    // 1. Resolve by TargetModel
    let gpt4o = registry.get_for_model(TargetModel::Gpt4o).unwrap();
    assert_eq!(gpt4o.name(), "o200k_base");

    let claude = registry.get_for_model(TargetModel::Claude35Sonnet).unwrap();
    assert_eq!(claude.name(), "claude_bpe");

    let llama = registry.get_for_model(TargetModel::Llama3).unwrap();
    assert_eq!(llama.name(), "llama3_bpe");

    // 2. Resolve by string identifier
    let o200k = registry.get_by_name("o200k_base").unwrap();
    assert_eq!(o200k.target_model(), TargetModel::Gpt4o);

    let cl100k = registry.get_by_name("cl100k_base").unwrap();
    assert_eq!(cl100k.target_model(), TargetModel::Gpt4Turbo);

    // 3. Resolve by model family
    let google_default = registry.default_for_family(ModelFamily::Google);
    assert_eq!(google_default.name(), "gemini_sp");

    // 4. Unknown returns None
    assert!(registry.get_by_name("non_existent_tokenizer").is_none());
}

#[test]
fn test_context_window_and_budget_ceilings() {
    let models = [
        TargetModel::Gpt4o,
        TargetModel::Gpt4Turbo,
        TargetModel::Claude35Sonnet,
        TargetModel::Claude3Haiku,
        TargetModel::Gemini15Pro,
        TargetModel::Gemini15Flash,
        TargetModel::Llama3,
        TargetModel::GenericFallback,
    ];

    for model in models {
        let window = model.context_window_limit();
        let ceiling = model.recommended_budget_ceiling();
        let max_output = model.max_output_tokens();

        assert!(window > 0);
        assert!(ceiling > 0);
        assert!(ceiling < window);
        assert!(max_output > 0);
    }
}

#[test]
fn test_document_token_enrichment_and_summary() {
    let mut doc = build_sample_cdm_document();
    let adapter = OpenAiAdapter::for_model(TargetModel::Gpt4o);

    let summary = enrich_document_token_metrics(&mut doc, &adapter);

    assert_eq!(summary.document_id, doc.document_id);
    assert_eq!(summary.total_sections, 3);
    assert_eq!(summary.tokenizer_name, "o200k_base");
    assert!(summary.total_tokens > 0);
    assert!(summary.total_characters > 0);
    assert!(summary.total_words > 0);
    assert!(summary.average_chars_per_token > 0.0);
    assert!(!summary.exceeds_context_window);

    // Verify each section has updated token_metrics
    for sec in &doc.sections {
        let metrics = sec
            .token_metrics
            .as_ref()
            .expect("section token metrics present");
        assert_eq!(metrics.tokenizer_name.as_deref(), Some("o200k_base"));
        assert!(metrics.exact_tokens.is_some());
        assert_eq!(metrics.exact_tokens.unwrap(), metrics.estimated_tokens);
    }

    // Markdown summary rendering
    let md = summary.to_markdown_summary();
    assert!(md.contains("### Tokenization & Context Budget Summary"));
    assert!(md.contains(&format!("`{}`", doc.document_id)));
    assert!(md.contains("- **Tokenizer:** `o200k_base`"));
    assert!(md.contains("Document fits safely within target model context window."));

    // JSON serialization roundtrip
    let json = serde_json::to_string_pretty(&summary).expect("valid json serialization");
    let deserialized: DocumentTokenSummary =
        serde_json::from_str(&json).expect("valid json deserialization");
    assert_eq!(summary, deserialized);
}

#[test]
fn test_section_token_metrics_individual_counting() {
    let sec = make_test_section("s-single", "test.rs", "fn compute() -> u32 { 42 }");
    let adapter = CalibratedHeuristicAdapter::default();

    let metrics = count_section_tokens(&sec, &adapter);
    assert!(metrics.estimated_tokens > 0);
    assert!(metrics.exact_tokens.is_none());
    assert_eq!(
        metrics.tokenizer_name.as_deref(),
        Some("calibrated_heuristic")
    );
}
