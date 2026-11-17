import { describe, it, expect } from "vitest";
import {
  calculateSavingsPct,
  calculateUtilizationPct,
  calculateHeadroom,
  evaluateBudgetUtilization,
} from "../src/lib/types/token-budget";
import { TokenBudgetBar, JobWorkspace } from "../src/lib/components/workspace/index";
import type { JobTokenMetrics } from "../src/lib/types/job";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("Token Budget Math & Utilization Rules", () => {
  describe("calculateSavingsPct", () => {
    it("returns 0 when raw tokens is zero or negative", () => {
      expect(calculateSavingsPct(0, 0)).toBe(0);
      expect(calculateSavingsPct(-10, 5)).toBe(0);
    });

    it("returns 0 when final tokens equals or exceeds raw tokens", () => {
      expect(calculateSavingsPct(1000, 1000)).toBe(0);
      expect(calculateSavingsPct(1000, 1200)).toBe(0);
    });

    it("calculates correct savings percentage", () => {
      expect(calculateSavingsPct(100000, 70000)).toBe(30);
      expect(calculateSavingsPct(100000, 50000)).toBe(50);
      expect(calculateSavingsPct(74500, 48900)).toBe(34);
    });
  });

  describe("calculateUtilizationPct", () => {
    it("returns 0 when budget limit is zero or negative", () => {
      expect(calculateUtilizationPct(50000, 0)).toBe(0);
      expect(calculateUtilizationPct(50000, -100)).toBe(0);
    });

    it("calculates utilization within budget", () => {
      expect(calculateUtilizationPct(50000, 100000)).toBe(50);
      expect(calculateUtilizationPct(80000, 100000)).toBe(80);
    });

    it("calculates utilization when exceeding budget", () => {
      expect(calculateUtilizationPct(150000, 100000)).toBe(150);
    });
  });

  describe("calculateHeadroom", () => {
    it("calculates positive remaining headroom", () => {
      expect(calculateHeadroom(40000, 100000)).toBe(60000);
      expect(calculateHeadroom(95000, 100000)).toBe(5000);
    });

    it("clamps to zero when budget is exceeded", () => {
      expect(calculateHeadroom(120000, 100000)).toBe(0);
    });
  });

  describe("evaluateBudgetUtilization", () => {
    it("evaluates optimal status when well within budget", () => {
      const metrics: JobTokenMetrics = {
        rawTokens: 80000,
        deduplicatedTokens: 55000,
        finalTokens: 50000,
        budgetLimit: 160000,
        savingsPct: 38,
        budgetRatio: 0.31,
      };

      const report = evaluateBudgetUtilization(metrics, "claude-3.5-sonnet");
      expect(report.status).toBe("optimal");
      expect(report.utilizationPct).toBe(31);
      expect(report.headroomTokens).toBe(110000);
      expect(report.savingsTokens).toBe(30000);
      expect(report.modelContextWindow).toBe(200000);
    });

    it("evaluates warning status when utilization is between 85% and 100%", () => {
      const metrics: JobTokenMetrics = {
        rawTokens: 180000,
        deduplicatedTokens: 155000,
        finalTokens: 145000,
        budgetLimit: 160000,
        savingsPct: 19,
        budgetRatio: 0.91,
      };

      const report = evaluateBudgetUtilization(metrics, "claude-3.5-sonnet");
      expect(report.status).toBe("warning");
      expect(report.utilizationPct).toBe(91);
      expect(report.statusMessage).toContain("91%");
    });

    it("evaluates exceeded status when final tokens exceed budget ceiling", () => {
      const metrics: JobTokenMetrics = {
        rawTokens: 300000,
        deduplicatedTokens: 250000,
        finalTokens: 220000,
        budgetLimit: 160000,
        savingsPct: 27,
        budgetRatio: 1.38,
      };

      const report = evaluateBudgetUtilization(metrics, "claude-3.5-sonnet");
      expect(report.status).toBe("exceeded");
      expect(report.utilizationPct).toBe(138);
      expect(report.headroomTokens).toBe(0);
      expect(report.statusMessage).toContain("Exceeds context budget by 60,000 tokens");
    });

    it("falls back to Claude 3.5 Sonnet defaults when unknown preset is provided", () => {
      const metrics: JobTokenMetrics = {
        rawTokens: 20000,
        deduplicatedTokens: 15000,
        finalTokens: 10000,
        budgetLimit: 0,
        savingsPct: 50,
        budgetRatio: 0.05,
      };

      const report = evaluateBudgetUtilization(metrics, "unknown-model-xyz");
      expect(report.modelContextWindow).toBe(200000);
      expect(report.budgetLimit).toBe(160000);
    });
  });
});

describe("TokenBudgetBar Component Invariants & Accessibility", () => {
  it("exports TokenBudgetBar from workspace module", () => {
    expect(TokenBudgetBar).toBeDefined();
  });

  it("validates TokenBudgetBar.svelte ARIA contracts, landmarks and controls", () => {
    const sveltePath = resolve(process.cwd(), "src/lib/components/workspace/TokenBudgetBar.svelte");
    const content = readFileSync(sveltePath, "utf-8");

    // Region landmark
    expect(content).toContain('role="region"');
    expect(content).toContain('aria-label="Token Statistics and Context Budget Bar"');

    // Progressbar landmark
    expect(content).toContain('role="progressbar"');
    expect(content).toContain("aria-valuenow");
    expect(content).toContain("aria-valuemin={0}");
    expect(content).toContain("aria-valuemax={100}");
    expect(content).toContain('aria-label="Context window token budget utilization"');

    // Alert landmark when budget exceeded
    expect(content).toContain('role="alert"');

    // 4 metric cards
    expect(content).toContain("Raw Ingested Tokens");
    expect(content).toContain("Deduplicated Tokens");
    expect(content).toContain("Final Context Output");
    expect(content).toContain("Token Optimization Savings");

    // Model preset switcher
    expect(content).toContain("handleSelectPreset");
    expect(content).toContain("Gemini 2M");
    expect(content).toContain("Claude 200k");
    expect(content).toContain("GPT-4o 128k");

    // Safety margin indicator
    expect(content).toContain("80% safety margin");
  });

  it("validates JobWorkspace.svelte integrates TokenBudgetBar", () => {
    const workspacePath = resolve(
      process.cwd(),
      "src/lib/components/workspace/JobWorkspace.svelte"
    );
    const content = readFileSync(workspacePath, "utf-8");

    expect(content).toContain("TokenBudgetBar");
    expect(content).toContain("bind:metrics={job.tokenMetrics}");
    expect(content).toContain("bind:modelPreset={job.modelPreset}");
  });
});
