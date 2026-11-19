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

export type ExtensionMessage =
  | { type: "PING" }
  | { type: "CHECK_BACKEND_STATUS"; apiUrl?: string; apiKey?: string }
  | { type: "GET_CONFIG" }
  | { type: "SAVE_CONFIG"; config: Partial<ExtensionConfig> }
  | { type: "RESET_CONFIG" }
  | { type: "GET_ACTIVE_TAB" }
  | { type: "DETECT_URL_TYPE"; url: string }
  | {
      type: "START_CONVERSION";
      url: string;
      format: OutputFormat;
      tokenBudget?: number | null;
      model?: TargetModel;
    };

export interface ResponseSuccess<T = unknown> {
  success: true;
  data: T;
}

export interface ResponseError {
  success: false;
  error: string;
}

export type ExtensionResponse<T = unknown> = ResponseSuccess<T> | ResponseError;
