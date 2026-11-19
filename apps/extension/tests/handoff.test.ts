import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { copyToClipboard, formatContentForClipboard } from "../src/handoff/clipboard";
import {
  buildPreFilledUrl,
  buildPreviewUrl,
  buildWorkspaceUrl,
  navigateToUrl,
} from "../src/handoff/deeplink";
import {
  getExtensionForFormat,
  getMimeTypeForFormat,
  sanitizeDownloadFilename,
  triggerFileDownload,
} from "../src/handoff/download";
import { setupMockChrome } from "./mocks/chrome";

describe("Result Handoff Subsystem (apps/extension)", () => {
  let mockChrome: ReturnType<typeof setupMockChrome>;

  beforeEach(() => {
    mockChrome = setupMockChrome();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("formatContentForClipboard", () => {
    const sampleContent = "# Article Title\n\nParagraph text here.";
    const sampleUrl = "https://example.com/blog/ai";
    const sampleTitle = "AI Trends 2026";

    it("formats raw content when mode is 'raw'", () => {
      const result = formatContentForClipboard(sampleContent, sampleUrl, sampleTitle, "markdown", {
        mode: "raw",
      });
      expect(result).toBe(sampleContent);
    });

    it("formats structured AI prompt with code fence and context headers", () => {
      const result = formatContentForClipboard(sampleContent, sampleUrl, sampleTitle, "markdown", {
        mode: "prompt",
      });

      expect(result).toContain("# Context: AI Trends 2026");
      expect(result).toContain("Source: https://example.com/blog/ai");
      expect(result).toContain("Format: MARKDOWN");
      expect(result).toContain("```markdown\n# Article Title\n\nParagraph text here.\n```");
      expect(result).toContain("Please use the reference context above");
    });

    it("formats markdown link when mode is 'markdown_link'", () => {
      const result = formatContentForClipboard(sampleContent, sampleUrl, sampleTitle, "markdown", {
        mode: "markdown_link",
      });
      expect(result).toBe("[AI Trends 2026](https://example.com/blog/ai)");
    });

    it("formats URL only when mode is 'url_only'", () => {
      const result = formatContentForClipboard(sampleContent, sampleUrl, sampleTitle, "markdown", {
        mode: "url_only",
      });
      expect(result).toBe("https://example.com/blog/ai");
    });
  });

  describe("copyToClipboard", () => {
    it("returns false for empty text", async () => {
      expect(await copyToClipboard("")).toBe(false);
    });

    it("uses navigator.clipboard.writeText when available", async () => {
      const writeTextMock = vi.fn().mockResolvedValue(undefined);
      (globalThis as any).navigator = {
        clipboard: { writeText: writeTextMock },
      };

      const success = await copyToClipboard("Hello world");
      expect(success).toBe(true);
      expect(writeTextMock).toHaveBeenCalledWith("Hello world");
    });

    it("uses document.execCommand fallback when navigator.clipboard fails", async () => {
      const writeTextMock = vi.fn().mockRejectedValue(new Error("Permission denied"));
      const execCommandMock = vi.fn().mockReturnValue(true);

      (globalThis as any).navigator = {
        clipboard: { writeText: writeTextMock },
      };
      (globalThis as any).document = {
        createElement: () => ({
          style: {},
          focus: vi.fn(),
          select: vi.fn(),
        }),
        body: {
          appendChild: vi.fn(),
          removeChild: vi.fn(),
        },
        execCommand: execCommandMock,
      };

      const success = await copyToClipboard("Fallback content");
      expect(success).toBe(true);
      expect(execCommandMock).toHaveBeenCalledWith("copy");
    });
  });

  describe("download utilities", () => {
    it("returns correct MIME types and extensions for all formats", () => {
      expect(getMimeTypeForFormat("markdown")).toContain("text/markdown");
      expect(getExtensionForFormat("markdown")).toBe("md");

      expect(getMimeTypeForFormat("txt")).toContain("text/plain");
      expect(getExtensionForFormat("txt")).toBe("txt");

      expect(getMimeTypeForFormat("json")).toContain("application/json");
      expect(getExtensionForFormat("json")).toBe("json");

      expect(getMimeTypeForFormat("html")).toContain("text/html");
      expect(getExtensionForFormat("html")).toBe("html");

      expect(getMimeTypeForFormat("jsonl")).toContain("application/x-ndjson");
      expect(getExtensionForFormat("jsonl")).toBe("jsonl");

      expect(getMimeTypeForFormat("zip")).toBe("application/zip");
      expect(getExtensionForFormat("zip")).toBe("zip");
    });

    it("sanitizes filenames from URLs and titles", () => {
      const filename1 = sanitizeDownloadFilename(
        "https://github.com/rust-lang/rust",
        "0193425b-7b24-7000-8000-000000000001",
        "markdown"
      );
      expect(filename1).toMatch(/^github-com-rust-lang-rust-0193425b\.md$/);

      const filename2 = sanitizeDownloadFilename(
        "Documentation: Intro to Tokio (v1.0)",
        "0193425b-7b24-7000-8000-000000000002",
        "zip"
      );
      expect(filename2).not.toContain(":");
      expect(filename2).not.toContain("(");
      expect(filename2).toMatch(/0193425b\.zip$/);
    });

    it("triggers file download via chrome.downloads if available", async () => {
      const mockDownload = vi.fn().mockResolvedValue(42);
      (globalThis as any).chrome = {
        ...mockChrome,
        downloads: { download: mockDownload },
      };
      (globalThis as any).Blob = class {
        constructor(
          public parts: any[],
          public opts: any
        ) {}
      };
      (globalThis as any).URL = {
        createObjectURL: () => "blob:chrome-ext://1234",
        revokeObjectURL: vi.fn(),
      };

      await triggerFileDownload("# Content", "test-doc.md", "markdown");

      expect(mockDownload).toHaveBeenCalledWith(
        expect.objectContaining({
          url: "blob:chrome-ext://1234",
          filename: "test-doc.md",
        })
      );
    });
  });

  describe("deeplink utilities", () => {
    const baseUrl = "http://localhost:3000";

    it("builds workspace URL with jobId query parameter", () => {
      const url = buildWorkspaceUrl(baseUrl, "job-xyz-123");
      expect(url).toBe("http://localhost:3000/workspace?job_id=job-xyz-123");
    });

    it("builds preview URL with optional format parameter", () => {
      const urlWithFmt = buildPreviewUrl(baseUrl, "job-xyz-123", "json");
      expect(urlWithFmt).toBe("http://localhost:3000/jobs/job-xyz-123/preview?format=json");

      const urlWithoutFmt = buildPreviewUrl(baseUrl, "job-xyz-123");
      expect(urlWithoutFmt).toBe("http://localhost:3000/jobs/job-xyz-123/preview");
    });

    it("builds pre-filled URL with source and format", () => {
      const url = buildPreFilledUrl(baseUrl, "https://github.com/sveltejs/svelte", "markdown");
      expect(url).toContain("http://localhost:3000/?");
      expect(url).toContain("source=https%3A%2F%2Fgithub.com%2Fsveltejs%2Fsvelte");
      expect(url).toContain("format=markdown");
    });

    it("navigates using chrome.tabs.create", async () => {
      const createMock = vi.fn().mockResolvedValue({ id: 999 });
      (globalThis as any).chrome = {
        tabs: { create: createMock },
      };

      await navigateToUrl("http://localhost:3000/workspace?job_id=123");
      expect(createMock).toHaveBeenCalledWith({
        url: "http://localhost:3000/workspace?job_id=123",
        active: true,
      });
    });
  });
});
