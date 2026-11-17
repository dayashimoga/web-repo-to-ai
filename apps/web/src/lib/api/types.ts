/**
 * REST API Client Data Transfer Objects & Problem Details (`apps/web/src/lib/api/types.ts`)
 *
 * Synchronized with:
 * - `services/api/src/dto/jobs.rs`
 * - `services/api/src/dto/sources.rs`
 * - `docs/API.md`
 */

export type ApiJobStatus = "queued" | "processing" | "completed" | "failed" | "cancelled";

export interface CreateJobPayload {
  /** One or more target URLs or repository identifiers. */
  sources: string[];
  /** Target Git branch, tag, or commit reference for repository sources. */
  git_ref?: string;
  /** Optional file glob inclusion/exclusion patterns. */
  path_patterns?: string[];
  /** Custom token budget ceiling (1,000 to 2,000,000 tokens). */
  token_budget?: number;
  /** Target LLM model identifier for budgeting (e.g. "claude-3-5-sonnet"). */
  target_model?: string;
  /** Formats to generate (e.g. ["markdown", "json", "html", "zip"]). */
  export_formats?: string[];
  /** Whether to perform exact file-level content deduplication. */
  deduplicate_exact?: boolean;
  /** Whether to perform semantic near-duplicate suppression. */
  deduplicate_semantic?: boolean;
  /** Whether to strip code comments during normalization. */
  remove_comments?: boolean;
}

export interface ApiJobResponse {
  /** Strongly-typed UUID v7 job identifier. */
  job_id: string;
  /** Current execution status. */
  status: ApiJobStatus;
  /** List of source URLs being processed. */
  sources: string[];
  /** Effective token budget applied to this job. */
  token_budget: number;
  /** Target LLM model architecture for token counting. */
  target_model: string;
  /** Configured export formats. */
  export_formats: string[];
  /** UTC timestamp of job creation (ISO-8601). */
  created_at: string;
}

export type SourceClassification = "web_page" | "public_github" | "private_github" | "batch_list";

export interface DetectedSource {
  url: string;
  classification: SourceClassification;
  normalized_url: string;
  repo_owner?: string;
  repo_name?: string;
  git_ref?: string;
  subpath?: string;
}

export interface DetectSourcesPayload {
  urls: string[];
}

export interface DetectSourcesResponse {
  detected: DetectedSource[];
  total_detected: number;
}

export interface InvalidParam {
  name: string;
  reason: string;
}

/** RFC 7807 Problem Details representation. */
export interface ApiProblemDetails {
  type: string;
  title: string;
  status: number;
  detail: string;
  instance?: string;
  error_code?: string;
  request_id?: string;
  invalid_params?: InvalidParam[];
}

export interface PreviewFileItem {
  id: string;
  path: string;
  language: string;
  byte_size: number;
  token_count: number;
  line_count: number;
}

export interface JobPreviewResponse {
  job_id: string;
  format: string;
  content: string;
  total_lines: number;
  total_tokens: number;
  total_bytes: number;
  truncated: boolean;
  max_lines: number;
  files: PreviewFileItem[];
  hierarchy?: unknown;
}
