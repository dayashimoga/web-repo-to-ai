import { beforeEach, describe, expect, it } from "vitest";
import {
  CONFIG_STORAGE_KEY,
  loadConfig,
  resetConfig,
  saveConfig,
  validateConfig,
} from "../src/storage/config";
import { DEFAULT_CONFIG } from "../src/types/index";
import { setupMockChrome } from "./mocks/chrome";

describe("Configuration Storage & Validation (apps/extension)", () => {
  let mockChrome: ReturnType<typeof setupMockChrome>;

  beforeEach(() => {
    mockChrome = setupMockChrome();
  });

  describe("validateConfig", () => {
    it("accepts valid default configuration", () => {
      const result = validateConfig(DEFAULT_CONFIG);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("rejects empty or whitespace-only apiUrl", () => {
      const result = validateConfig({ apiUrl: "   " });
      expect(result.valid).toBe(false);
      expect(result.errors[0]).toMatch(/cannot be empty/i);
    });

    it("rejects invalid URL protocols", () => {
      const ftpResult = validateConfig({ apiUrl: "ftp://example.com/api" });
      expect(ftpResult.valid).toBe(false);
      expect(ftpResult.errors[0]).toMatch(/http or https protocol/i);

      const fileResult = validateConfig({ apiUrl: "file:///C:/secrets" });
      expect(fileResult.valid).toBe(false);
    });

    it("rejects completely malformed URLs", () => {
      const result = validateConfig({ apiUrl: "not-a-valid-url" });
      expect(result.valid).toBe(false);
      expect(result.errors[0]).toMatch(/valid, well-formed URL/i);
    });

    it("rejects unsupported export formats", () => {
      const result = validateConfig({ defaultFormat: "pdf" as any });
      expect(result.valid).toBe(false);
      expect(result.errors[0]).toMatch(/invalid format 'pdf'/i);
    });

    it("rejects unsupported target models", () => {
      const result = validateConfig({ defaultTargetModel: "llama-3" as any });
      expect(result.valid).toBe(false);
      expect(result.errors[0]).toMatch(/invalid target model 'llama-3'/i);
    });

    it("accepts positive token budget or null", () => {
      expect(validateConfig({ defaultTokenBudget: 64000 }).valid).toBe(true);
      expect(validateConfig({ defaultTokenBudget: null }).valid).toBe(true);
    });

    it("rejects negative or zero token budget", () => {
      expect(validateConfig({ defaultTokenBudget: 0 }).valid).toBe(false);
      expect(validateConfig({ defaultTokenBudget: -500 }).valid).toBe(false);
    });
  });

  describe("loadConfig", () => {
    it("returns DEFAULT_CONFIG when storage is empty", async () => {
      const config = await loadConfig();
      expect(config).toEqual(DEFAULT_CONFIG);
    });

    it("merges stored overrides with DEFAULT_CONFIG", async () => {
      await mockChrome.storage.sync.set({
        [CONFIG_STORAGE_KEY]: {
          apiUrl: "https://custom-repo-ai.internal:8080",
          defaultFormat: "json",
        },
      });

      const config = await loadConfig();
      expect(config.apiUrl).toBe("https://custom-repo-ai.internal:8080");
      expect(config.defaultFormat).toBe("json");
      expect(config.defaultTokenBudget).toBe(DEFAULT_CONFIG.defaultTokenBudget);
      expect(config.defaultTargetModel).toBe(DEFAULT_CONFIG.defaultTargetModel);
    });

    it("trims whitespace from URLs and keys on load", async () => {
      await mockChrome.storage.sync.set({
        [CONFIG_STORAGE_KEY]: {
          apiUrl: "  http://localhost:4000/  ",
          apiKey: "  wr_live_secretkey123  ",
        },
      });

      const config = await loadConfig();
      expect(config.apiUrl).toBe("http://localhost:4000/");
      expect(config.apiKey).toBe("wr_live_secretkey123");
    });
  });

  describe("saveConfig", () => {
    it("validates and persists new configuration updates", async () => {
      const updated = await saveConfig({
        apiUrl: "http://127.0.0.1:9090",
        apiKey: "wr_live_test_api_key",
        defaultFormat: "html",
        defaultTokenBudget: 16000,
      });

      expect(updated.apiUrl).toBe("http://127.0.0.1:9090");
      expect(updated.apiKey).toBe("wr_live_test_api_key");
      expect(updated.defaultFormat).toBe("html");
      expect(updated.defaultTokenBudget).toBe(16000);

      // Verify persisted to storage
      const stored = await mockChrome.storage.sync.get(CONFIG_STORAGE_KEY);
      expect(stored[CONFIG_STORAGE_KEY].apiUrl).toBe("http://127.0.0.1:9090");
      expect(stored[CONFIG_STORAGE_KEY].defaultFormat).toBe("html");
    });

    it("rejects invalid updates and does not alter stored config", async () => {
      await saveConfig({ apiUrl: "http://localhost:3000" });

      await expect(
        saveConfig({ apiUrl: "invalid-url", defaultFormat: "unknown" as any })
      ).rejects.toThrow(/Configuration validation failed/i);

      const loaded = await loadConfig();
      expect(loaded.apiUrl).toBe("http://localhost:3000");
    });
  });

  describe("resetConfig", () => {
    it("resets modified configuration back to DEFAULT_CONFIG", async () => {
      await saveConfig({
        apiUrl: "https://modified.example.com",
        defaultFormat: "zip",
      });

      const reset = await resetConfig();
      expect(reset).toEqual(DEFAULT_CONFIG);

      const loaded = await loadConfig();
      expect(loaded).toEqual(DEFAULT_CONFIG);
    });
  });
});
