import { describe, it, expect } from "vitest";
import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { resolve, join } from "node:path";
import { highlightLines } from "../src/lib/utils/syntax-highlighter";
import { evaluateBudgetUtilization } from "../src/lib/types/token-budget";
import { detectUrlSource } from "../src/lib/utils/url-detector";
import { computeSha256Hex } from "../src/lib/types/download";
import {
  getContrastRatio,
  isWcagAaCompliant,
  isWcagAaaCompliant,
} from "../src/lib/tokens/color-utils";
import { colors } from "../src/lib/tokens/index";

function readComponentSource(relativePath: string): string {
  const filePath = resolve(process.cwd(), relativePath);
  return readFileSync(filePath, "utf-8");
}

describe("UI Performance, Bundle Budgets & Visual Regression Verification (Subphase 1.4.3)", () => {
  describe("1. Production Bundle Size & Zero-Runtime Dependency Budget Certification", () => {
    it("verifies package.json has ZERO external runtime dependencies", () => {
      const pkgPath = resolve(process.cwd(), "package.json");
      const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));

      // Production dependencies must be absent or strictly empty
      expect(pkg.dependencies === undefined || Object.keys(pkg.dependencies).length === 0).toBe(
        true
      );

      // Verify no heavy bloat libraries in devDependencies
      const devDeps = pkg.devDependencies || {};
      const bannedLibraries = [
        "lodash",
        "moment",
        "prismjs",
        "highlight.js",
        "rxjs",
        "lucide-react",
        "@mui/material",
        "@chakra-ui/react",
      ];

      for (const banned of bannedLibraries) {
        expect(
          devDeps[banned],
          `Banned dependency "${banned}" must not be installed`
        ).toBeUndefined();
      }
    });

    it("certifies client build asset footprint is within strict production budgets", () => {
      const clientDir = resolve(process.cwd(), ".svelte-kit/output/client");
      if (!existsSync(clientDir)) {
        // Build output verified when client directory exists
        return;
      }

      function getAllFiles(dir: string): string[] {
        let results: string[] = [];
        const list = readdirSync(dir);
        for (const file of list) {
          const filePath = join(dir, file);
          const stat = statSync(filePath);
          if (stat.isDirectory()) {
            results = results.concat(getAllFiles(filePath));
          } else {
            results.push(filePath);
          }
        }
        return results;
      }

      const allFiles = getAllFiles(clientDir);
      let totalClientBytes = 0;
      let maxJsChunkBytes = 0;
      let maxCssChunkBytes = 0;

      for (const file of allFiles) {
        const size = statSync(file).size;
        totalClientBytes += size;

        if (file.endsWith(".js")) {
          if (size > maxJsChunkBytes) maxJsChunkBytes = size;
        } else if (file.endsWith(".css")) {
          if (size > maxCssChunkBytes) maxCssChunkBytes = size;
        }
      }

      // Largest single JS chunk must be strictly <= 100 kB (raw)
      expect(maxJsChunkBytes).toBeLessThan(100 * 1024);

      // Largest CSS chunk must be strictly <= 60 kB (raw)
      expect(maxCssChunkBytes).toBeLessThan(60 * 1024);

      // Total client build footprint must be strictly <= 350 kB (raw)
      expect(totalClientBytes).toBeLessThan(350 * 1024);
    });
  });

  describe("2. Algorithmic Throughput & Micro-Benchmark Certification", () => {
    it("certifies Syntax Highlighter throughput (> 20,000 lines/sec on 1,000 JSON lines)", () => {
      const sampleJson = JSON.stringify(
        Array.from({ length: 200 }, (_, i) => ({
          id: `item-${i}`,
          index: i,
          title: `Document title ${i}`,
          active: i % 2 === 0,
          score: i * 1.5,
          tags: ["typescript", "svelte", "rust"],
        })),
        null,
        2
      );

      const t0 = performance.now();
      const lines = highlightLines(sampleJson, "json");
      const durationMs = performance.now() - t0;

      expect(lines.length).toBeGreaterThanOrEqual(1000);
      expect(durationMs).toBeLessThan(50); // Under 50ms for 1000+ lines

      const throughputLinesPerSec = (lines.length / durationMs) * 1000;
      expect(throughputLinesPerSec).toBeGreaterThan(20000);
    });

    it("certifies XML Syntax Highlighter throughput on 500+ XML lines", () => {
      const sampleXml = Array.from(
        { length: 150 },
        (_, i) =>
          `  <document index="${i}" path="src/module_${i}.rs" tokens="256">\n    <content>fn execute() { println!("item ${i}"); }</content>\n  </document>`
      ).join("\n");

      const xmlDoc = `<documents count="150">\n${sampleXml}\n</documents>`;

      const t0 = performance.now();
      const lines = highlightLines(xmlDoc, "xml");
      const durationMs = performance.now() - t0;

      expect(lines.length).toBeGreaterThanOrEqual(400);
      expect(durationMs).toBeLessThan(40);
    });

    it("certifies Token Budget Evaluation engine execution (10,000 calls in < 50ms)", () => {
      const metrics = {
        rawTokens: 125000,
        deduplicatedTokens: 98000,
        finalTokens: 85000,
        savingsTokens: 40000,
        savingsPct: 32.0,
        budgetLimit: 200000,
        utilizationPct: 42.5,
        budgetRatio: 0.425,
      };

      const t0 = performance.now();
      for (let i = 0; i < 10000; i++) {
        evaluateBudgetUtilization(metrics, "claude-3.5-sonnet");
      }
      const durationMs = performance.now() - t0;

      expect(durationMs).toBeLessThan(50);
    });

    it("certifies URL Detection & SSRF Pre-check throughput (5,000 calls in < 50ms)", () => {
      const testUrls = [
        "https://github.com/dayashimoga/web-repo-to-ai",
        "https://docs.rs/serde/latest/serde/",
        "https://example.com/sitemap.xml",
        "http://127.0.0.1:8080/admin",
        "http://169.254.169.254/latest/meta-data/",
      ];

      const t0 = performance.now();
      for (let i = 0; i < 1000; i++) {
        for (const url of testUrls) {
          detectUrlSource(url);
        }
      }
      const durationMs = performance.now() - t0;

      expect(durationMs).toBeLessThan(50);
    });

    it("certifies Web Crypto SHA-256 Digest throughput on 500 KB test payload in < 50ms", async () => {
      const payload = "A".repeat(500 * 1024); // 500 KB payload

      const t0 = performance.now();
      const hash = await computeSha256Hex(payload);
      const durationMs = performance.now() - t0;

      expect(hash).toHaveLength(64);
      expect(durationMs).toBeLessThan(50);
    });
  });

  describe("3. Visual Design System & Styling Token Regression Invariants", () => {
    it("verifies tokens.css defines all required design system custom properties", () => {
      const tokensCss = readComponentSource("src/lib/styles/tokens.css");

      // Brand palette
      expect(tokensCss).toContain("--color-brand-500: #6366f1;");
      expect(tokensCss).toContain("--color-brand-600: #4f46e5;");

      // Accent palette
      expect(tokensCss).toContain("--color-accent-400: #22d3ee;");
      expect(tokensCss).toContain("--color-accent-500: #06b6d4;");

      // Surface palette
      expect(tokensCss).toContain("--color-surface-50: #f8fafc;");
      expect(tokensCss).toContain("--color-surface-900: #0f172a;");
      expect(tokensCss).toContain("--color-surface-950: #090d16;");

      // Semantic status
      expect(tokensCss).toContain("--color-status-success: #10b981;");
      expect(tokensCss).toContain("--color-status-warning: #f59e0b;");
      expect(tokensCss).toContain("--color-status-error: #ef4444;");

      // Elevation shadows
      expect(tokensCss).toContain("--elevation-1:");
      expect(tokensCss).toContain("--elevation-2:");
      expect(tokensCss).toContain("--elevation-3:");

      // Dark mode overrides
      expect(tokensCss).toContain(".dark");
      expect(tokensCss).toContain("--theme-bg-canvas: var(--color-surface-950);");
      expect(tokensCss).toContain("--theme-bg-panel: var(--color-surface-900);");
      expect(tokensCss).toContain("--theme-text-primary: var(--color-surface-50);");
    });

    it("verifies app.css defines semantic theme utility classes", () => {
      const appCss = readComponentSource("src/lib/styles/app.css");

      expect(appCss).toContain(".bg-theme-canvas");
      expect(appCss).toContain(".bg-theme-panel");
      expect(appCss).toContain(".bg-theme-subtle");
      expect(appCss).toContain(".text-theme-primary");
      expect(appCss).toContain(".text-theme-secondary");
      expect(appCss).toContain(".text-theme-muted");
      expect(appCss).toContain(".border-theme-subtle");
      expect(appCss).toContain(".border-theme-strong");
    });

    it("certifies WCAG AAA contrast compliance across both Light and Dark theme states", () => {
      // Dark mode: Light text on Dark canvas
      const darkRatio = getContrastRatio("#f8fafc", colors.surface[950]);
      expect(darkRatio).toBeGreaterThanOrEqual(14.0);
      expect(isWcagAaaCompliant("#f8fafc", colors.surface[950])).toBe(true);

      // Dark mode: Light text on Dark panel
      const darkPanelRatio = getContrastRatio("#f8fafc", colors.surface[900]);
      expect(darkPanelRatio).toBeGreaterThanOrEqual(12.0);
      expect(isWcagAaaCompliant("#f8fafc", colors.surface[900])).toBe(true);

      // Light mode: Dark text on White panel
      const lightRatio = getContrastRatio(colors.surface[900], "#ffffff");
      expect(lightRatio).toBeGreaterThanOrEqual(14.0);
      expect(isWcagAaaCompliant(colors.surface[900], "#ffffff")).toBe(true);

      // Brand primary button: White on Brand-600
      const brandRatio = getContrastRatio("#ffffff", colors.brand[600]);
      expect(brandRatio).toBeGreaterThanOrEqual(4.5);
      expect(isWcagAaCompliant("#ffffff", colors.brand[600])).toBe(true);
    });

    it("verifies components do not contain arbitrary hardcoded hex color inline styles", () => {
      const componentsToAudit = [
        "src/lib/components/navigation/Header.svelte",
        "src/lib/components/navigation/Footer.svelte",
        "src/lib/components/landing/Hero.svelte",
        "src/lib/components/landing/WorkflowSteps.svelte",
        "src/lib/components/landing/SupportedSources.svelte",
        "src/lib/components/landing/FeatureHighlights.svelte",
        "src/lib/components/input/UrlInput.svelte",
        "src/lib/components/settings/ConversionSettings.svelte",
        "src/lib/components/workspace/JobWorkspace.svelte",
        "src/lib/components/workspace/TokenBudgetBar.svelte",
        "src/lib/components/workspace/PreviewPanel.svelte",
        "src/lib/components/workspace/DownloadControls.svelte",
      ];

      for (const compPath of componentsToAudit) {
        const content = readComponentSource(compPath);
        // Assert no style="color: #..." or style="background-color: #..."
        expect(content).not.toMatch(/style=["'][^"']*(?:color|background):\s*#[0-9a-fA-F]{3,8}/i);
      }
    });
  });
});
