// Configuration storage helper for web-repo-to-ai browser extension

import { validateApiBaseUrl } from "../security/policy";
import { DEFAULT_CONFIG, ExtensionConfig, OutputFormat, TargetModel } from "../types/index";

export const CONFIG_STORAGE_KEY = "web_repo_to_ai_config";

const VALID_FORMATS: ReadonlyArray<OutputFormat> = [
  "markdown",
  "txt",
  "json",
  "html",
  "jsonl",
  "zip",
];

const VALID_MODELS: ReadonlyArray<TargetModel> = [
  "gpt-4o",
  "claude-3-5-sonnet",
  "gemini-1-5-pro",
  "generic",
];

/**
 * Validates extension configuration options.
 */
export function validateConfig(config: Partial<ExtensionConfig>): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (config.apiUrl !== undefined) {
    const valResult = validateApiBaseUrl(config.apiUrl);
    if (!valResult.valid && valResult.reason) {
      errors.push(valResult.reason);
    }
  }

  if (config.defaultFormat !== undefined) {
    if (!VALID_FORMATS.includes(config.defaultFormat)) {
      errors.push(`Invalid format '${config.defaultFormat}'. Allowed: ${VALID_FORMATS.join(", ")}`);
    }
  }

  if (config.defaultTargetModel !== undefined) {
    if (!VALID_MODELS.includes(config.defaultTargetModel)) {
      errors.push(
        `Invalid target model '${config.defaultTargetModel}'. Allowed: ${VALID_MODELS.join(", ")}`
      );
    }
  }

  if (config.defaultTokenBudget !== undefined && config.defaultTokenBudget !== null) {
    if (
      typeof config.defaultTokenBudget !== "number" ||
      isNaN(config.defaultTokenBudget) ||
      config.defaultTokenBudget <= 0
    ) {
      errors.push("Token budget must be a positive number or null (unlimited).");
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Internal storage accessor that detects chrome.storage.sync or chrome.storage.local
 * or falls back to an in-memory store in non-extension environments.
 */
function getStorageArea(): {
  get: (key: string) => Promise<Record<string, unknown>>;
  set: (items: Record<string, unknown>) => Promise<void>;
  remove: (key: string) => Promise<void>;
} {
  if (
    typeof chrome !== "undefined" &&
    chrome.storage &&
    chrome.storage.sync &&
    typeof chrome.storage.sync.get === "function"
  ) {
    return {
      get: (key: string) =>
        new Promise((resolve, reject) => {
          try {
            chrome.storage.sync.get(key, (items) => {
              if (chrome.runtime && chrome.runtime.lastError) {
                // If sync fails (e.g. quota or offline), fallback to local
                chrome.storage.local.get(key, (localItems) => {
                  resolve(localItems || {});
                });
              } else {
                resolve(items || {});
              }
            });
          } catch (err) {
            reject(err);
          }
        }),
      set: (items: Record<string, unknown>) =>
        new Promise((resolve, reject) => {
          try {
            chrome.storage.sync.set(items, () => {
              if (chrome.runtime && chrome.runtime.lastError) {
                chrome.storage.local.set(items, () => resolve());
              } else {
                resolve();
              }
            });
          } catch (err) {
            reject(err);
          }
        }),
      remove: (key: string) =>
        new Promise((resolve) => {
          try {
            chrome.storage.sync.remove(key, () => {
              chrome.storage.local.remove(key, () => resolve());
            });
          } catch {
            resolve();
          }
        }),
    };
  }

  if (
    typeof chrome !== "undefined" &&
    chrome.storage &&
    chrome.storage.local &&
    typeof chrome.storage.local.get === "function"
  ) {
    return {
      get: (key: string) =>
        new Promise((resolve) => {
          chrome.storage.local.get(key, (items) => resolve(items || {}));
        }),
      set: (items: Record<string, unknown>) =>
        new Promise((resolve) => {
          chrome.storage.local.set(items, () => resolve());
        }),
      remove: (key: string) =>
        new Promise((resolve) => {
          chrome.storage.local.remove(key, () => resolve());
        }),
    };
  }

  // In-memory fallback for test environments without chrome API
  const memoryStore: Record<string, unknown> = {};
  return {
    get: async (key: string) => {
      const val = memoryStore[key];
      return val ? { [key]: val } : {};
    },
    set: async (items: Record<string, unknown>) => {
      Object.assign(memoryStore, items);
    },
    remove: async (key: string) => {
      delete memoryStore[key];
    },
  };
}

/**
 * Loads the current extension configuration, applying defaults for missing keys.
 */
export async function loadConfig(): Promise<ExtensionConfig> {
  const storage = getStorageArea();
  try {
    const result = await storage.get(CONFIG_STORAGE_KEY);
    const stored = (result[CONFIG_STORAGE_KEY] || {}) as Partial<ExtensionConfig>;

    return {
      apiUrl: stored.apiUrl?.trim() || DEFAULT_CONFIG.apiUrl,
      apiKey: stored.apiKey !== undefined ? stored.apiKey.trim() : DEFAULT_CONFIG.apiKey,
      defaultFormat: stored.defaultFormat || DEFAULT_CONFIG.defaultFormat,
      defaultTokenBudget:
        stored.defaultTokenBudget !== undefined
          ? stored.defaultTokenBudget
          : DEFAULT_CONFIG.defaultTokenBudget,
      defaultTargetModel: stored.defaultTargetModel || DEFAULT_CONFIG.defaultTargetModel,
      autoDetectSource:
        stored.autoDetectSource !== undefined
          ? stored.autoDetectSource
          : DEFAULT_CONFIG.autoDetectSource,
      includeFrontmatter:
        stored.includeFrontmatter !== undefined
          ? stored.includeFrontmatter
          : DEFAULT_CONFIG.includeFrontmatter,
      sendDomSnapshot:
        stored.sendDomSnapshot !== undefined
          ? stored.sendDomSnapshot
          : DEFAULT_CONFIG.sendDomSnapshot,
    };
  } catch (error) {
    console.warn("Failed to load stored extension configuration, falling back to defaults", error);
    return { ...DEFAULT_CONFIG };
  }
}

/**
 * Validates and saves updated configuration options.
 */
export async function saveConfig(updates: Partial<ExtensionConfig>): Promise<ExtensionConfig> {
  const validation = validateConfig(updates);
  if (!validation.valid) {
    throw new Error(`Configuration validation failed: ${validation.errors.join("; ")}`);
  }

  const current = await loadConfig();
  const merged: ExtensionConfig = {
    ...current,
    ...updates,
    apiUrl: updates.apiUrl !== undefined ? updates.apiUrl.trim() : current.apiUrl,
    apiKey: updates.apiKey !== undefined ? updates.apiKey.trim() : current.apiKey,
  };

  const storage = getStorageArea();
  await storage.set({ [CONFIG_STORAGE_KEY]: merged });
  return merged;
}

/**
 * Resets the configuration back to default values.
 */
export async function resetConfig(): Promise<ExtensionConfig> {
  const storage = getStorageArea();
  await storage.set({ [CONFIG_STORAGE_KEY]: DEFAULT_CONFIG });
  return { ...DEFAULT_CONFIG };
}
