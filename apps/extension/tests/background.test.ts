import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  checkBackendStatus,
  classifyUrl,
  getActiveTab,
  handleMessage,
  updateActionBadge,
} from "../src/background/index";
import { DEFAULT_CONFIG } from "../src/types/index";
import { setupMockChrome } from "./mocks/chrome";

describe("Background Service Worker (apps/extension)", () => {
  let mockChrome: ReturnType<typeof setupMockChrome>;

  beforeEach(() => {
    mockChrome = setupMockChrome();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("classifyUrl", () => {
    it("handles invalid, empty, or non-string inputs", () => {
      expect(classifyUrl("").isSupported).toBe(false);
      expect(classifyUrl("not-a-valid-url").isSupported).toBe(false);
      expect(classifyUrl(null as any).isSupported).toBe(false);
    });

    it("rejects browser internal pages", () => {
      const chromeSettings = classifyUrl("chrome://settings");
      expect(chromeSettings.isSupported).toBe(false);
      expect(chromeSettings.archetype).toBe("unknown");

      const edgeFlags = classifyUrl("edge://flags");
      expect(edgeFlags.isSupported).toBe(false);

      const aboutBlank = classifyUrl("about:blank");
      expect(aboutBlank.isSupported).toBe(false);
    });

    it("accurately classifies GitHub repositories", () => {
      const result = classifyUrl("https://github.com/rust-lang/rust");
      expect(result.archetype).toBe("github_repository");
      expect(result.isSupported).toBe(true);
      expect(result.displayName).toBe("GitHub: rust-lang/rust");
      expect(result.details?.owner).toBe("rust-lang");
      expect(result.details?.repo).toBe("rust");

      const subpathResult = classifyUrl(
        "https://github.com/facebook/react/tree/main/packages/react"
      );
      expect(subpathResult.archetype).toBe("github_repository");
      expect(subpathResult.details?.subpath).toBe("tree/main/packages/react");
    });

    it("does not classify non-repo GitHub pages as repositories", () => {
      const settings = classifyUrl("https://github.com/settings/profile");
      expect(settings.archetype).not.toBe("github_repository");

      const explore = classifyUrl("https://github.com/explore");
      expect(explore.archetype).not.toBe("github_repository");
    });

    it("accurately classifies technical documentation sites", () => {
      expect(classifyUrl("https://docs.rs/tokio/latest").archetype).toBe("web_documentation");
      expect(classifyUrl("https://svelte.dev/docs/kit/introduction").archetype).toBe(
        "web_documentation"
      );
      expect(classifyUrl("https://myproject.readthedocs.io/en/latest/").archetype).toBe(
        "web_documentation"
      );
    });

    it("accurately classifies web articles and blogs", () => {
      expect(classifyUrl("https://medium.com/@engineer/building-rust-clis").archetype).toBe(
        "web_article"
      );
      expect(classifyUrl("https://dev.to/user/web-assembly-guide").archetype).toBe("web_article");
      expect(classifyUrl("https://company.org/blog/2026-release-notes").archetype).toBe(
        "web_article"
      );
    });

    it("accurately classifies structured data files", () => {
      expect(classifyUrl("https://example.com/api/openapi.json").archetype).toBe("file_resource");
      expect(classifyUrl("https://example.com/config.yaml").archetype).toBe("file_resource");
      expect(classifyUrl("https://example.com/dataset.csv").archetype).toBe("file_resource");
    });

    it("classifies general websites as generic_web", () => {
      const res = classifyUrl("https://news.ycombinator.com/");
      expect(res.archetype).toBe("generic_web");
      expect(res.isSupported).toBe(true);
    });
  });

  describe("checkBackendStatus & updateActionBadge", () => {
    it("reports healthy connection and clears badge when API is reachable", async () => {
      const mockFetch = vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        statusText: "OK",
        json: async () => ({ status: "healthy", version: "0.1.0" }),
      });
      globalThis.fetch = mockFetch;

      const result = await checkBackendStatus("http://localhost:3000", "test-token");
      expect(result.connected).toBe(true);
      expect(result.version).toBe("0.1.0");
      expect(result.status).toBe("healthy");
      expect(mockChrome.action.badgeText).toBe("");
    });

    it("reports offline and sets ERR badge when server returns error", async () => {
      const mockFetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 503,
        statusText: "Service Unavailable",
      });
      globalThis.fetch = mockFetch;

      const result = await checkBackendStatus("http://localhost:3000");
      expect(result.connected).toBe(false);
      expect(result.error).toMatch(/HTTP 503/i);
      expect(mockChrome.action.badgeText).toBe("ERR");
      expect(mockChrome.action.badgeBackgroundColor).toBe("#ef4444");
    });

    it("reports offline when network throws error", async () => {
      const mockFetch = vi.fn().mockRejectedValue(new Error("ECONNREFUSED"));
      globalThis.fetch = mockFetch;

      const result = await checkBackendStatus("http://localhost:3000");
      expect(result.connected).toBe(false);
      expect(result.error).toMatch(/ECONNREFUSED/i);
      expect(mockChrome.action.badgeText).toBe("ERR");
    });
  });

  describe("getActiveTab", () => {
    it("returns the current active tab from chrome.tabs.query", async () => {
      const tab = await getActiveTab();
      expect(tab).not.toBeNull();
      expect(tab?.id).toBe(101);
      expect(tab?.url).toBe("https://github.com/rust-lang/rust");
      expect(tab?.title).toMatch(/rust-lang\/rust/i);
    });

    it("returns null if no active tabs are found", async () => {
      mockChrome.tabs.mockTabs = [];
      const tab = await getActiveTab();
      expect(tab).toBeNull();
    });
  });

  describe("handleMessage", () => {
    it("handles PING message", async () => {
      const response = await handleMessage({ type: "PING" });
      expect(response.success).toBe(true);
      if (response.success) {
        expect((response.data as any).status).toBe("pong");
      }
    });

    it("handles GET_CONFIG and SAVE_CONFIG messages", async () => {
      const getRes = await handleMessage({ type: "GET_CONFIG" });
      expect(getRes.success).toBe(true);
      if (getRes.success) {
        expect(getRes.data).toEqual(DEFAULT_CONFIG);
      }

      const saveRes = await handleMessage({
        type: "SAVE_CONFIG",
        config: { defaultFormat: "jsonl", defaultTokenBudget: 64000 },
      });
      expect(saveRes.success).toBe(true);
      if (saveRes.success) {
        expect((saveRes.data as any).defaultFormat).toBe("jsonl");
        expect((saveRes.data as any).defaultTokenBudget).toBe(64000);
      }
    });

    it("handles GET_ACTIVE_TAB and DETECT_URL_TYPE", async () => {
      const tabRes = await handleMessage({ type: "GET_ACTIVE_TAB" });
      expect(tabRes.success).toBe(true);
      if (tabRes.success) {
        expect((tabRes.data as any).url).toBe("https://github.com/rust-lang/rust");
      }

      const detectRes = await handleMessage({
        type: "DETECT_URL_TYPE",
        url: "https://svelte.dev/docs/kit/introduction",
      });
      expect(detectRes.success).toBe(true);
      if (detectRes.success) {
        expect((detectRes.data as any).archetype).toBe("web_documentation");
      }
    });

    it("handles CAPTURE_DOM message", async () => {
      const res = await handleMessage({ type: "CAPTURE_DOM" });
      expect(res.success).toBe(true);
      if (res.success) {
        expect((res.data as any).capturedAt).toBeDefined();
      }
    });

    it("handles START_CONVERSION with valid URL and returns ConversionResult", async () => {
      // Mock fetch for createJob, poll, fetchArtifact
      globalThis.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          ok: true,
          status: 202,
          json: async () => ({
            job_id: "job-bg-123",
            status: "queued",
            sources: ["https://github.com/rust-lang/rust"],
          }),
        })
        .mockResolvedValueOnce({
          ok: true,
          status: 200,
          json: async () => ({
            job_id: "job-bg-123",
            status: "completed",
          }),
        })
        .mockResolvedValueOnce({
          ok: true,
          status: 200,
          text: async () => "# Converted Rust Codebase",
        });

      const convertRes = await handleMessage({
        type: "START_CONVERSION",
        url: "https://github.com/rust-lang/rust",
        format: "markdown",
        tokenBudget: 32000,
      });

      expect(convertRes.success).toBe(true);
      if (convertRes.success) {
        const data = convertRes.data as any;
        expect(data.jobId).toBe("job-bg-123");
        expect(data.content).toBe("# Converted Rust Codebase");
        expect(data.format).toBe("markdown");
      }

      // Check status query
      const statusRes = await handleMessage({ type: "GET_CONVERSION_STATUS" });
      expect(statusRes.success).toBe(true);
      if (statusRes.success) {
        expect((statusRes.data as any).stage).toBe("completed");
      }
    });

    it("rejects START_CONVERSION with unsupported URL", async () => {
      const convertRes = await handleMessage({
        type: "START_CONVERSION",
        url: "chrome://extensions",
        format: "markdown",
      });

      expect(convertRes.success).toBe(false);
      expect((convertRes as any).error).toMatch(/not supported for conversion/i);
    });

    it("returns error for unknown message type", async () => {
      const res = await handleMessage({ type: "UNKNOWN_COMMAND" } as any);
      expect(res.success).toBe(false);
      expect((res as any).error).toMatch(/Unhandled message type/i);
    });
  });
});
