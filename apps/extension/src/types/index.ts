// Shared TypeScript Type Definitions for web-repo-to-ai Browser Extension

export type OutputFormat = "markdown" | "txt" | "json" | "html" | "jsonl" | "zip";

export type TargetModel = "gpt-4o" | "claude-3-5-sonnet" | "gemini-1-5-pro" | "generic";

export type DetectedArchetype =
  | "github_repository"
  | "web_documentation"
  | "web_article"
  | "file_resource"
  | "generic_web"
  | "local_directory"
  | "unknown";

export interface ExtensionConfig {
  apiUrl: string;
  apiKey: string;
  defaultFormat: OutputFormat;
  defaultTokenBudget: number | null;
  defaultTargetModel: TargetModel;
  autoDetectSource: boolean;
  includeFrontmatter: boolean;
  sendDomSnapshot: boolean;
}

export const DEFAULT_CONFIG: Readonly<ExtensionConfig> = Object.freeze({
  apiUrl: "http://localhost:3000",
  apiKey: "",
  defaultFormat: "markdown",
  defaultTokenBudget: 32000,
  defaultTargetModel: "gpt-4o",
  autoDetectSource: true,
  includeFrontmatter: true,
  sendDomSnapshot: false,
});

export interface BackendStatus {
  connected: boolean;
  version?: string;
  status?: string;
  latencyMs?: number;
  error?: string;
}

export interface SourceClassification {
  archetype: DetectedArchetype;
  displayName: string;
  url: string;
  isSupported: boolean;
  details?: Record<string, unknown>;
}

export interface TabInfo {
  id?: number;
  url: string;
  title: string;
  favIconUrl?: string;
}

export interface PageHeading {
  level: number;
  text: string;
}

export interface PageSnapshot {
  url: string;
  title: string;
  html: string;
  textContent: string;
  charCount: number;
  estimatedTokens: number;
  canonicalUrl?: string;
  description?: string;
  author?: string;
  headings: PageHeading[];
  capturedAt: string;
}

export type ConversionStage =
  "idle" | "capturing" | "submitting" | "processing" | "fetching_result" | "completed" | "error";

export interface ConversionResult {
  jobId: string;
  url: string;
  format: OutputFormat;
  content: string;
  tokenCount: number;
  charCount: number;
  durationMs: number;
  archetype: DetectedArchetype;
  downloadUrl?: string;
}

export interface ConversionProgress {
  stage: ConversionStage;
  progressPct: number;
  message: string;
  jobId?: string;
  error?: string;
  result?: ConversionResult;
}

export type ExtensionMessage =
  | { type: "PING" }
  | { type: "CHECK_BACKEND_STATUS"; apiUrl?: string; apiKey?: string }
  | { type: "GET_CONFIG" }
  | { type: "SAVE_CONFIG"; config: Partial<ExtensionConfig> }
  | { type: "RESET_CONFIG" }
  | { type: "GET_ACTIVE_TAB" }
  | { type: "DETECT_URL_TYPE"; url: string }
  | { type: "CAPTURE_DOM"; tabId?: number }
  | {
      type: "START_CONVERSION";
      url: string;
      format: OutputFormat;
      tokenBudget?: number | null;
      model?: TargetModel;
      sendDomSnapshot?: boolean;
      tabId?: number;
    }
  | { type: "GET_CONVERSION_STATUS"; jobId?: string };

export interface ResponseSuccess<T = unknown> {
  success: true;
  data: T;
}

export interface ResponseError {
  success: false;
  error: string;
}

export type ExtensionResponse<T = unknown> = ResponseSuccess<T> | ResponseError;
