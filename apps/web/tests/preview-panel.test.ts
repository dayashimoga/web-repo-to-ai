import { describe, it, expect } from "vitest";
import {
  PREVIEW_FORMATS,
  estimateTokens,
  formatContentForExport,
  generateSampleExtractedFiles,
  type PreviewFile,
} from "../src/lib/types/preview";
import { escapeHtml, highlightSyntax, highlightLines } from "../src/lib/utils/syntax-highlighter";
import { PreviewPanel, JobWorkspace } from "../src/lib/components/workspace/index";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("Preview Panel Domain Types & Formatters", () => {
  it("defines the 5 core export formats with valid metadata", () => {
    expect(PREVIEW_FORMATS).toHaveLength(5);
    const ids = PREVIEW_FORMATS.map((f) => f.id);
    expect(ids).toContain("markdown");
    expect(ids).toContain("json");
    expect(ids).toContain("xml");
    expect(ids).toContain("txt");
    expect(ids).toContain("jsonl");

    PREVIEW_FORMATS.forEach((fmt) => {
      expect(fmt.badge).toMatch(/^\.[a-z]+$/);
      expect(fmt.mime).toBeTruthy();
      expect(fmt.description).toBeTruthy();
    });
  });

  it("estimates token counts with deterministic heuristic", () => {
    expect(estimateTokens("")).toBe(0);
    const shortText = "hello world";
    expect(estimateTokens(shortText)).toBeGreaterThan(0);
    const longText = "a".repeat(380);
    expect(estimateTokens(longText)).toBe(100);
  });

  it("generates sample extracted files for GitHub repos and web pages", () => {
    const repoFiles = generateSampleExtractedFiles("https://github.com/dayashimoga/web-repo-to-ai");
    expect(repoFiles.length).toBeGreaterThanOrEqual(3);
    expect(repoFiles.map((f) => f.path)).toContain("README.md");
    expect(repoFiles.map((f) => f.path)).toContain("Cargo.toml");

    const webFiles = generateSampleExtractedFiles("https://docs.rs/tokio/latest/tokio/");
    expect(webFiles).toHaveLength(1);
    expect(webFiles[0].path).toBe("article.md");
  });

  describe("formatContentForExport", () => {
    const mockFiles: PreviewFile[] = [
      {
        id: "f1",
        path: "src/main.rs",
        language: "rust",
        sizeBytes: 85,
        tokenCount: 22,
        content: 'fn main() {\n    println!("Hello, World!");\n}',
      },
      {
        id: "f2",
        path: "README.md",
        language: "markdown",
        sizeBytes: 42,
        tokenCount: 11,
        content: "# Test Readme\nSample project.",
      },
    ];

    it("formats valid Markdown export with metadata and fences", () => {
      const output = formatContentForExport(mockFiles, "markdown", {
        sourceUrl: "https://github.com/test/repo",
      });

      expect(output).toContain("# Context Export: https://github.com/test/repo");
      expect(output).toContain("## File: `src/main.rs`");
      expect(output).toContain("```rust");
      expect(output).toContain("fn main()");
      expect(output).toContain("## File: `README.md`");
      expect(output).toContain("```markdown");
    });

    it("formats valid JSON Canonical Data Model (CDM) matching schema v1.0.0", () => {
      const output = formatContentForExport(mockFiles, "json", {
        sourceUrl: "https://github.com/test/repo",
      });

      const parsed = JSON.parse(output);
      expect(parsed.schema_version).toBe("1.0.0");
      expect(parsed.source_url).toBe("https://github.com/test/repo");
      expect(parsed.total_files).toBe(2);
      expect(parsed.sections).toHaveLength(2);
      expect(parsed.sections[0].original_path).toBe("src/main.rs");
      expect(parsed.completeness.is_complete).toBe(true);
      expect(parsed.completeness.discovered_count).toBe(2);
    });

    it("formats valid Claude XML document hierarchy", () => {
      const output = formatContentForExport(mockFiles, "xml", {
        sourceUrl: "https://github.com/test/repo",
      });

      expect(output).toContain('<?xml version="1.0" encoding="UTF-8"?>');
      expect(output).toContain('<documents source="https://github.com/test/repo"');
      expect(output).toContain('<document index="1">');
      expect(output).toContain("<source>src/main.rs</source>");
      expect(output).toContain("<document_content>");
      expect(output).toContain("fn main()");
      expect(output).toContain("</document_content>");
      expect(output).toContain("</documents>");
    });

    it("formats clean Plaintext with ASCII demarcations", () => {
      const output = formatContentForExport(mockFiles, "txt", {
        sourceUrl: "https://github.com/test/repo",
      });

      expect(output).toContain("WEB-REPO-TO-AI CONTEXT EXPORT");
      expect(output).toContain("FILE: src/main.rs [RUST]");
      expect(output).toContain("FILE: README.md [MARKDOWN]");
    });

    it("formats JSONL with one valid JSON object per line", () => {
      const output = formatContentForExport(mockFiles, "jsonl", {
        sourceUrl: "https://github.com/test/repo",
      });

      const lines = output.trim().split("\n");
      expect(lines).toHaveLength(2);

      const obj1 = JSON.parse(lines[0]);
      expect(obj1.path).toBe("src/main.rs");
      expect(obj1.language).toBe("rust");

      const obj2 = JSON.parse(lines[1]);
      expect(obj2.path).toBe("README.md");
      expect(obj2.language).toBe("markdown");
    });

    it("handles empty files gracefully", () => {
      const output = formatContentForExport([], "markdown");
      expect(output).toContain("# Empty Output");
    });
  });
});

describe("Syntax Highlighter & Tokenizer Engine", () => {
  it("escapes dangerous HTML characters preventing XSS", () => {
    const dirty = '<script>alert("XSS & danger")</script>';
    const safe = escapeHtml(dirty);
    expect(safe).not.toContain("<script>");
    expect(safe).toContain("&lt;script&gt;");
    expect(safe).toContain("&amp;");
    expect(safe).toContain("&quot;");
  });

  it("highlights JSON syntax tokens", () => {
    const jsonStr = '{\n  "name": "web-repo-to-ai",\n  "count": 42,\n  "enabled": true\n}';
    const highlighted = highlightSyntax(jsonStr, "json");

    expect(highlighted).toContain("text-brand-400"); // Key
    expect(highlighted).toContain("text-green-400"); // String value
    expect(highlighted).toContain("text-amber-400"); // Number
    expect(highlighted).toContain("text-accent-400"); // Boolean
  });

  it("highlights XML tags and attributes", () => {
    const xmlStr = '<document index="1"><source>README.md</source></document>';
    const highlighted = highlightSyntax(xmlStr, "xml");

    expect(highlighted).toContain("text-brand-400"); // Tag name
    expect(highlighted).toContain("text-accent-400"); // Attribute name
    expect(highlighted).toContain("text-green-400"); // Attribute value
  });

  it("highlights Markdown headings, blockquotes, and lists", () => {
    const mdStr = "# Heading 1\n> A quote\n- An item";
    const highlighted = highlightSyntax(mdStr, "markdown");

    expect(highlighted).toContain("text-brand-400"); // Heading
    expect(highlighted).toContain("text-surface-400"); // Blockquote
  });

  it("splits code into numbered lines with highlightLines", () => {
    const code = "line 1\nline 2\nline 3";
    const lines = highlightLines(code, "text");

    expect(lines).toHaveLength(3);
    expect(lines[0].lineNumber).toBe(1);
    expect(lines[1].lineNumber).toBe(2);
    expect(lines[2].lineNumber).toBe(3);
  });
});

describe("PreviewPanel Component Invariants & Accessibility", () => {
  it("exports PreviewPanel from workspace module", () => {
    expect(PreviewPanel).toBeDefined();
  });

  it("validates PreviewPanel.svelte ARIA contracts, landmarks and controls", () => {
    const sveltePath = resolve(process.cwd(), "src/lib/components/workspace/PreviewPanel.svelte");
    const content = readFileSync(sveltePath, "utf-8");

    // Region landmark
    expect(content).toContain('role="region"');
    expect(content).toContain('aria-label="Content Preview Panel"');

    // Tablist & tabs
    expect(content).toContain('role="tablist"');
    expect(content).toContain('role="tab"');
    expect(content).toContain("aria-selected");
    expect(content).toContain("aria-controls");

    // Tabpanel
    expect(content).toContain('role="tabpanel"');

    // Live status region for copy feedback
    expect(content).toContain('role="status"');
    expect(content).toContain('aria-live="polite"');

    // Accessibility on line number gutter
    expect(content).toContain('aria-hidden="true"');

    // Interactive action buttons
    expect(content).toContain("handleCopy");
    expect(content).toContain("handleDownload");
    expect(content).toContain("showLineNumbers");
    expect(content).toContain("wrapText");
  });

  it("validates JobWorkspace.svelte integrates PreviewPanel", () => {
    const workspacePath = resolve(
      process.cwd(),
      "src/lib/components/workspace/JobWorkspace.svelte"
    );
    const content = readFileSync(workspacePath, "utf-8");

    expect(content).toContain("PreviewPanel");
    expect(content).toContain("isPreviewVisible");
    expect(content).toContain("getPreviewFormat");
    expect(content).toContain("Extracted Content & Multi-Format Preview");
  });
});
