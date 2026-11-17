import { describe, it, expect } from "vitest";
import {
  createDefaultPipelineStages,
  createInitialJob,
  verifyCompletenessInvariant,
  calculateOverallProgress,
  type PipelineStageInfo,
  type CompletenessReport,
} from "../src/lib/types/job";
import { JobWorkspace } from "../src/lib/components/workspace/index";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("Conversion Job Domain Model & State Machine", () => {
  it("creates the standard 4-stage pipeline in correct sequence", () => {
    const stages = createDefaultPipelineStages();
    expect(stages).toHaveLength(4);

    expect(stages[0].id).toBe("acquisition");
    expect(stages[0].name).toContain("Acquisition");
    expect(stages[0].state).toBe("pending");
    expect(stages[0].progressPct).toBe(0);

    expect(stages[1].id).toBe("normalization");
    expect(stages[1].name).toContain("Canonicalization");
    expect(stages[1].state).toBe("pending");
    expect(stages[1].progressPct).toBe(0);

    expect(stages[2].id).toBe("deduplication");
    expect(stages[2].name).toContain("Deduplication");
    expect(stages[2].state).toBe("pending");
    expect(stages[2].progressPct).toBe(0);

    expect(stages[3].id).toBe("export");
    expect(stages[3].name).toContain("Export");
    expect(stages[3].state).toBe("pending");
    expect(stages[3].progressPct).toBe(0);
  });

  it("creates initial job with deterministic defaults and initial logging", () => {
    const job = createInitialJob("https://github.com/dayashimoga/web-repo-to-ai");

    expect(job.id).toBeDefined();
    expect(job.sourceUrl).toBe("https://github.com/dayashimoga/web-repo-to-ai");
    expect(job.status).toBe("queued");
    expect(job.currentPhase).toBe("pending");
    expect(job.overallProgress).toBe(0);
    expect(job.targetFormat).toBe("markdown");
    expect(job.modelPreset).toBe("claude-3.5-sonnet");
    expect(job.tokenMetrics.budgetLimit).toBe(160000);
    expect(job.stages).toHaveLength(4);
    expect(job.logs).toHaveLength(1);
    expect(job.logs[0].message).toContain("https://github.com/dayashimoga/web-repo-to-ai");
  });

  it("honors custom options in createInitialJob", () => {
    const job = createInitialJob("https://docs.rs/tokio/latest/tokio/", {
      id: "custom-job-42",
      detectedSource: "Documentation Hub",
      targetFormat: "json",
      modelPreset: "gemini-1.5-pro",
      budgetLimit: 500000,
    });

    expect(job.id).toBe("custom-job-42");
    expect(job.sourceUrl).toBe("https://docs.rs/tokio/latest/tokio/");
    expect(job.detectedSource).toBe("Documentation Hub");
    expect(job.targetFormat).toBe("json");
    expect(job.modelPreset).toBe("gemini-1.5-pro");
    expect(job.tokenMetrics.budgetLimit).toBe(500000);
  });

  describe("Completeness Accounting Invariant: discovered = extracted + excluded + failed", () => {
    it("verifies valid zero state", () => {
      const validZero: CompletenessReport = {
        discoveredCount: 0,
        extractedCount: 0,
        excludedCount: 0,
        failedCount: 0,
        isComplete: true,
      };
      expect(verifyCompletenessInvariant(validZero)).toBe(true);
    });

    it("verifies valid non-zero completed state", () => {
      const validReport: CompletenessReport = {
        discoveredCount: 150,
        extractedCount: 135,
        excludedCount: 12,
        failedCount: 3,
        isComplete: true,
      };
      expect(verifyCompletenessInvariant(validReport)).toBe(true);
      expect(135 + 12 + 3).toBe(150);
    });

    it("flags accounting discrepancy when files are silently dropped", () => {
      const discrepancyReport: CompletenessReport = {
        discoveredCount: 150,
        extractedCount: 130, // 8 files silently dropped!
        excludedCount: 10,
        failedCount: 2,
        isComplete: false,
      };
      expect(verifyCompletenessInvariant(discrepancyReport)).toBe(false);
    });
  });

  describe("Weighted Overall Progress Calculation", () => {
    it("returns 0 for empty stages array", () => {
      expect(calculateOverallProgress([])).toBe(0);
    });

    it("returns 0 when all stages are pending", () => {
      const stages = createDefaultPipelineStages();
      expect(calculateOverallProgress(stages)).toBe(0);
    });

    it("calculates weighted progress when first stage is running at 50%", () => {
      const stages = createDefaultPipelineStages();
      stages[0].state = "in_progress";
      stages[0].progressPct = 50;

      // 50 / 4 = 12.5 -> rounds to 13
      expect(calculateOverallProgress(stages)).toBe(13);
    });

    it("calculates 25% when first stage is completed and remaining pending", () => {
      const stages = createDefaultPipelineStages();
      stages[0].state = "completed";
      stages[0].progressPct = 100;

      // 100 / 4 = 25
      expect(calculateOverallProgress(stages)).toBe(25);
    });

    it("calculates 100% when all stages are completed", () => {
      const stages = createDefaultPipelineStages().map((s) => ({
        ...s,
        state: "completed" as const,
        progressPct: 100,
      }));

      expect(calculateOverallProgress(stages)).toBe(100);
    });
  });
});

describe("Workspace Component Invariants & Accessibility", () => {
  it("exports JobWorkspace from workspace index module", () => {
    expect(JobWorkspace).toBeDefined();
  });

  it("validates JobWorkspace.svelte source has required ARIA landmarks and controls", () => {
    const sveltePath = resolve(process.cwd(), "src/lib/components/workspace/JobWorkspace.svelte");
    const content = readFileSync(sveltePath, "utf-8");

    // ARIA progressbar landmark
    expect(content).toContain('role="progressbar"');
    expect(content).toContain("aria-valuenow");
    expect(content).toContain("aria-valuemin");
    expect(content).toContain("aria-valuemax");

    // Telemetry log window with aria-live polite
    expect(content).toContain('role="log"');
    expect(content).toContain('aria-live="polite"');

    // Completeness accounting matrix
    expect(content).toContain("Repository Completeness Accounting");
    expect(content).toContain("discovered = extracted + excluded + failed");

    // Failure alert banner
    expect(content).toContain('role="alert"');

    // Interactive action buttons
    expect(content).toContain("handleCancel");
    expect(content).toContain("handleRetry");

    // Bounded log slice (Rule 03 memory bounds)
    expect(content).toContain("job.logs.slice(-200)");
  });

  it("validates /workspace/+page.svelte route integration", () => {
    const routePath = resolve(process.cwd(), "src/routes/workspace/+page.svelte");
    const content = readFileSync(routePath, "utf-8");

    expect(content).toContain("JobWorkspace");
    expect(content).toContain("createInitialJob");
    expect(content).toContain("Header");
    expect(content).toContain("Footer");
    expect(content).toContain("detectUrlSource");
    expect(content).toContain('<main id="main-content"');
  });
});
