/**
 * Conversion Settings Contracts & Calibrated AI Model Presets
 *
 * Defines configuration options for:
 * 1. Target AI model context windows (Gemini, Claude, GPT-4o, Local LLMs).
 * 2. Token budget allocations and ceiling thresholds.
 * 3. CDM normalization and filtering flags (boilerplate, comments, tests).
 * 4. Ingestion controls (dynamic Chromium fallback, file size boundaries).
 */

export type AIModelPresetId =
  | "gemini-1.5-pro"
  | "gemini-1.5-flash"
  | "claude-3.5-sonnet"
  | "gpt-4o"
  | "local-128k"
  | "local-32k"
  | "custom";

export interface AIModelPreset {
  id: AIModelPresetId;
  name: string;
  provider: string;
  contextWindow: number;
  defaultBudget: number;
  description: string;
  badge: string;
}

export const AI_MODEL_PRESETS: Record<AIModelPresetId, AIModelPreset> = {
  "gemini-1.5-pro": {
    id: "gemini-1.5-pro",
    name: "Gemini 1.5 Pro",
    provider: "Google",
    contextWindow: 2000000,
    defaultBudget: 1000000,
    description: "2M token window capable of holding entire monorepos and multi-volume docs",
    badge: "2M Tokens",
  },
  "gemini-1.5-flash": {
    id: "gemini-1.5-flash",
    name: "Gemini 1.5 Flash",
    provider: "Google",
    contextWindow: 1000000,
    defaultBudget: 500000,
    description: "1M fast inference budget for medium to large codebases and articles",
    badge: "1M Tokens",
  },
  "claude-3.5-sonnet": {
    id: "claude-3.5-sonnet",
    name: "Claude 3.5 Sonnet",
    provider: "Anthropic",
    contextWindow: 200000,
    defaultBudget: 160000,
    description: "200k high-reasoning window with structured XML document blocks",
    badge: "200k Tokens",
  },
  "gpt-4o": {
    id: "gpt-4o",
    name: "GPT-4o",
    provider: "OpenAI",
    contextWindow: 128000,
    defaultBudget: 100000,
    description: "128k context window calibrated for OpenAI tiktoken encoding",
    badge: "128k Tokens",
  },
  "local-128k": {
    id: "local-128k",
    name: "Llama 3 / Mistral (128k)",
    provider: "Local / Ollama",
    contextWindow: 128000,
    defaultBudget: 96000,
    description: "128k extended context window for self-hosted local inference runners",
    badge: "Local 128k",
  },
  "local-32k": {
    id: "local-32k",
    name: "Local LLM (32k)",
    provider: "Local / Ollama",
    contextWindow: 32768,
    defaultBudget: 28000,
    description: "32k budget optimized for standard 8B/70B local quantized weights",
    badge: "Local 32k",
  },
  custom: {
    id: "custom",
    name: "Custom Budget",
    provider: "User Defined",
    contextWindow: 2000000,
    defaultBudget: 128000,
    description: "User-specified token ceiling from 8k to 2M tokens",
    badge: "Custom",
  },
};

export interface ConversionSettingsConfig {
  selectedPreset: AIModelPresetId;
  maxTokenBudget: number;
  stripBoilerplate: boolean;
  stripCodeComments: boolean;
  excludeTests: boolean;
  enableDynamicRendering: boolean;
  maxFileSizeKb: number;
  exactDeduplication: boolean;
  semanticDeduplication: boolean;
}

export function getDefaultConversionSettings(): ConversionSettingsConfig {
  return {
    selectedPreset: "claude-3.5-sonnet",
    maxTokenBudget: 160000,
    stripBoilerplate: true,
    stripCodeComments: false,
    excludeTests: true,
    enableDynamicRendering: false,
    maxFileSizeKb: 1024,
    exactDeduplication: true,
    semanticDeduplication: true,
  };
}

export function validateConversionSettings(settings: ConversionSettingsConfig): {
  isValid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (settings.maxTokenBudget < 1000) {
    errors.push("Token budget must be at least 1,000 tokens.");
  }
  if (settings.maxTokenBudget > 2000000) {
    errors.push("Token budget cannot exceed 2,000,000 tokens.");
  }
  if (settings.maxFileSizeKb < 64) {
    errors.push("Max file size must be at least 64 KB.");
  }
  if (settings.maxFileSizeKb > 10240) {
    errors.push("Max file size cannot exceed 10,240 KB (10 MB).");
  }

  return {
    isValid: errors.length === 0,
    errors,
  };
}
