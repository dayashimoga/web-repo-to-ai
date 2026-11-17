/**
 * Token Budget & Context Window Utilization Domain Model
 *
 * Implements:
 * 1. Mathematical calculation of token reduction, savings percentage, and headroom.
 * 2. Multi-model context window boundary validation (Gemini 2M, Claude 200k, GPT-4o 128k, Local 32k).
 * 3. Budget utilization state classifications (optimal, warning, exceeded).
 */

import { AI_MODEL_PRESETS, type AIModelPreset, type AIModelPresetId } from "./settings";
import type { JobTokenMetrics } from "./job";

export type BudgetStatusLevel = "optimal" | "warning" | "exceeded";

export interface BudgetUtilizationReport {
  rawTokens: number;
  deduplicatedTokens: number;
  finalTokens: number;
  budgetLimit: number;
  modelContextWindow: number;
  savingsTokens: number;
  savingsPct: number;
  utilizationPct: number;
  headroomTokens: number;
  status: BudgetStatusLevel;
  statusMessage: string;
}

/**
 * Calculates percentage savings between raw ingested tokens and final output tokens.
 */
export function calculateSavingsPct(rawTokens: number, finalTokens: number): number {
  if (rawTokens <= 0) return 0;
  if (finalTokens >= rawTokens) return 0;
  const savings = ((rawTokens - finalTokens) / rawTokens) * 100;
  return Math.min(100, Math.max(0, Math.round(savings)));
}

/**
 * Calculates budget utilization percentage relative to the configured budget limit.
 */
export function calculateUtilizationPct(finalTokens: number, budgetLimit: number): number {
  if (budgetLimit <= 0) return 0;
  const ratio = (finalTokens / budgetLimit) * 100;
  return Math.max(0, Math.round(ratio));
}

/**
 * Calculates available headroom tokens before reaching the budget ceiling.
 */
export function calculateHeadroom(finalTokens: number, budgetLimit: number): number {
  return Math.max(0, budgetLimit - finalTokens);
}

/**
 * Produces a full verified budget utilization report given token metrics and model preset.
 */
export function evaluateBudgetUtilization(
  metrics: JobTokenMetrics,
  presetId = "claude-3.5-sonnet"
): BudgetUtilizationReport {
  const typedId = presetId as AIModelPresetId;
  const preset: AIModelPreset = AI_MODEL_PRESETS[typedId] || AI_MODEL_PRESETS["claude-3.5-sonnet"];

  const budgetLimit = metrics.budgetLimit > 0 ? metrics.budgetLimit : preset.defaultBudget;
  const rawTokens = Math.max(0, metrics.rawTokens);
  const finalTokens = Math.max(0, metrics.finalTokens);
  const deduplicatedTokens = Math.max(finalTokens, metrics.deduplicatedTokens || finalTokens);

  const savingsTokens = Math.max(0, rawTokens - finalTokens);
  const savingsPct = calculateSavingsPct(rawTokens, finalTokens);
  const utilizationPct = calculateUtilizationPct(finalTokens, budgetLimit);
  const headroomTokens = calculateHeadroom(finalTokens, budgetLimit);

  let status: BudgetStatusLevel = "optimal";
  let statusMessage = "Fits comfortably within context window with safe reasoning buffer.";

  if (finalTokens > budgetLimit) {
    status = "exceeded";
    const overflow = finalTokens - budgetLimit;
    statusMessage = `Exceeds context budget by ${overflow.toLocaleString()} tokens! Switch to Gemini 1.5 Pro or filter files.`;
  } else if (utilizationPct > 85) {
    status = "warning";
    statusMessage = `Consuming ${utilizationPct}% of context budget. Reduced headroom available for model generation.`;
  }

  return {
    rawTokens,
    deduplicatedTokens,
    finalTokens,
    budgetLimit,
    modelContextWindow: preset.contextWindow,
    savingsTokens,
    savingsPct,
    utilizationPct,
    headroomTokens,
    status,
    statusMessage,
  };
}
