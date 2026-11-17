import { describe, it, expect } from "vitest";
import {
  computeSha256Hex,
  deriveArtifactBaseName,
  buildExportArtifacts,
  generateChecksumsTxt,
  generateChecksumManifestJson,
} from "../src/lib/types/download";
import { generateSampleExtractedFiles } from "../src/lib/types/preview";
import { DownloadControls, JobWorkspace } from "../src/lib/components/workspace/index";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("Download Controls Domain Types & Checksums", () => {
  describe("computeSha256Hex", () => {
    it("produces a deterministic 64-character hexadecimal SHA-256 hash", async () => {
      const text = "web-repo-to-ai zero-trust AI converter";
      const hash1 = await computeSha256Hex(text);
      const hash2 = await computeSha256Hex(text);

      expect(hash1).toHaveLength(64);
      expect(hash1).toMatch(/^[0-9a-f]{64}$/);
      expect(hash1).toBe(hash2);
    });

    it("generates distinct hashes for distinct contents", async () => {
      const hashA = await computeSha256Hex("Content A");
      const hashB = await computeSha256Hex("Content B");
      expect(hashA).not.toBe(hashB);
    });
  });

  describe("deriveArtifactBaseName", () => {
    it("derives repository name from GitHub URL", () => {
      expect(deriveArtifactBaseName("https://github.com/dayashimoga/web-repo-to-ai")).toBe(
        "web-repo-to-ai"
      );
      expect(deriveArtifactBaseName("https://github.com/tokio-rs/tokio")).toBe("tokio");
    });

    it("derives clean base name from documentation URLs", () => {
      expect(deriveArtifactBaseName("https://docs.rs/serde/latest/serde/")).toBe("serde");
    });

    it("falls back to domain name or context-export on abnormal URLs", () => {
      expect(deriveArtifactBaseName("https://example.com")).toBe("example_com");
      expect(deriveArtifactBaseName("invalid-url")).toBe("context-export");
    });
  });

  describe("buildExportArtifacts", () => {
    it("generates all 5 export format artifacts with real content and sha256 digests", async () => {
      const files = generateSampleExtractedFiles("https://github.com/dayashimoga/web-repo-to-ai");
      const artifacts = await buildExportArtifacts(
        files,
        "https://github.com/dayashimoga/web-repo-to-ai"
      );

      expect(artifacts).toHaveLength(5);
      const formats = artifacts.map((a) => a.format);
      expect(formats).toContain("markdown");
      expect(formats).toContain("json");
      expect(formats).toContain("xml");
      expect(formats).toContain("txt");
      expect(formats).toContain("jsonl");

      for (const art of artifacts) {
        expect(art.content.length).toBeGreaterThan(0);
        expect(art.byteSize).toBeGreaterThan(0);
        expect(art.sha256Hex).toHaveLength(64);
        expect(art.filename).toContain("web-repo-to-ai");
        expect(art.mimeType).toBeTruthy();
      }
    });
  });

  describe("generateChecksumsTxt and Manifest", () => {
    it("generates standard coreutils sha256sum format in checksums.txt", async () => {
      const files = generateSampleExtractedFiles("https://github.com/dayashimoga/web-repo-to-ai");
      const artifacts = await buildExportArtifacts(
        files,
        "https://github.com/dayashimoga/web-repo-to-ai"
      );
      const txt = generateChecksumsTxt(artifacts, "https://github.com/dayashimoga/web-repo-to-ai");

      expect(txt).toContain("# SHA-256 Checksums for web-repo-to-ai Context Export");
      expect(txt).toContain("web-repo-to-ai.md");
      expect(txt).toContain("web-repo-to-ai.json");

      const lines = txt.split("\n").filter((l) => l && !l.startsWith("#"));
      expect(lines).toHaveLength(5);
      lines.forEach((line) => {
        expect(line).toMatch(/^[0-9a-f]{64}\s\s[a-zA-Z0-9._-]+$/);
      });
    });

    it("generates schema-compliant checksum manifest JSON", async () => {
      const files = generateSampleExtractedFiles("https://github.com/dayashimoga/web-repo-to-ai");
      const artifacts = await buildExportArtifacts(
        files,
        "https://github.com/dayashimoga/web-repo-to-ai"
      );
      const jsonStr = generateChecksumManifestJson(
        artifacts,
        "https://github.com/dayashimoga/web-repo-to-ai"
      );

      const manifest = JSON.parse(jsonStr);
      expect(manifest.schema_version).toBe("1.0.0");
      expect(manifest.source_url).toBe("https://github.com/dayashimoga/web-repo-to-ai");
      expect(manifest.total_artifacts).toBe(5);
      expect(manifest.total_bytes).toBeGreaterThan(0);
      expect(manifest.artifacts).toHaveLength(5);
      expect(manifest.artifacts[0].sha256).toHaveLength(64);
    });
  });
});

describe("DownloadControls Component Invariants & Accessibility", () => {
  it("exports DownloadControls from workspace module", () => {
    expect(DownloadControls).toBeDefined();
  });

  it("validates DownloadControls.svelte ARIA contracts, landmarks and controls", () => {
    const sveltePath = resolve(
      process.cwd(),
      "src/lib/components/workspace/DownloadControls.svelte"
    );
    const content = readFileSync(sveltePath, "utf-8");

    // Region landmark
    expect(content).toContain('role="region"');
    expect(content).toContain('aria-label="Format Selection and Download Controls"');

    // Live status region for screen readers
    expect(content).toContain('role="status"');
    expect(content).toContain('aria-live="polite"');

    // Batch download actions
    expect(content).toContain("Download All Formats");
    expect(content).toContain("handleDownloadAll");
    expect(content).toContain("handleDownloadChecksumsTxt");
    expect(content).toContain("handleDownloadManifestJson");

    // Individual download actions
    expect(content).toContain("handleDownloadArtifact");
    expect(content).toContain("handleCopyChecksum");

    // Truncated hash display
    expect(content).toContain("truncateHash");
  });

  it("validates JobWorkspace.svelte integrates DownloadControls", () => {
    const workspacePath = resolve(
      process.cwd(),
      "src/lib/components/workspace/JobWorkspace.svelte"
    );
    const content = readFileSync(workspacePath, "utf-8");

    expect(content).toContain("DownloadControls");
    expect(content).toContain("sourceUrl={job.sourceUrl}");
  });
});
