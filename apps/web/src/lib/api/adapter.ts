/**
 * API Data Adapter (`apps/web/src/lib/api/adapter.ts`)
 *
 * Maps between backend API DTOs (`ApiJobResponse`) and the frontend domain model (`ConversionJob`).
 */

import type { ApiJobResponse, ApiJobStatus } from "./types";
import {
  type ConversionJob,
  type JobStatus,
  type PipelinePhase,
  createDefaultPipelineStages,
} from "$lib/types/job";

/**
 * Maps backend `ApiJobStatus` to frontend domain `JobStatus`.
 */
export function mapApiStatusToJobStatus(apiStatus: ApiJobStatus): JobStatus {
  switch (apiStatus) {
    case "queued":
      return "queued";
    case "processing":
      return "running";
    case "completed":
      return "completed";
    case "failed":
      return "failed";
    case "cancelled":
      return "cancelled";
    default:
      return "queued";
  }
}

/**
 * Maps backend job status to corresponding pipeline phase.
 */
export function mapStatusToPhase(status: JobStatus): PipelinePhase {
  switch (status) {
    case "queued":
      return "pending";
    case "running":
      return "acquisition";
    case "completed":
      return "completed";
    case "failed":
      return "acquisition";
    case "cancelled":
      return "pending";
    default:
      return "pending";
  }
}

/**
 * Transforms an `ApiJobResponse` into a complete `ConversionJob` domain model.
 */
export function mapApiJobToConversionJob(
  apiJob: ApiJobResponse,
  overrides?: {
    detectedSource?: string;
    targetFormat?: "markdown" | "json" | "xml" | "txt" | "jsonl" | "zip";
  }
): ConversionJob {
  const status = mapApiStatusToJobStatus(apiJob.status);
  const phase = mapStatusToPhase(status);
  const primarySource = apiJob.sources[0] || "";
  const primaryFormat = (overrides?.targetFormat || apiJob.export_formats[0] || "markdown") as
    "markdown" | "json" | "xml" | "txt" | "jsonl" | "zip";

  const stages = createDefaultPipelineStages();
  if (status === "running") {
    stages[0].state = "in_progress";
    stages[0].progressPct = 25;
  } else if (status === "completed") {
    for (const stage of stages) {
      stage.state = "completed";
      stage.progressPct = 100;
    }
  }

  const overallProgress = status === "completed" ? 100 : status === "running" ? 25 : 0;

  return {
    id: apiJob.job_id,
    sourceUrl: primarySource,
    detectedSource: overrides?.detectedSource || "Verified Source",
    targetFormat: primaryFormat,
    modelPreset: apiJob.target_model,
    status,
    currentPhase: phase,
    overallProgress,
    stages,
    completeness: {
      discoveredCount: 0,
      extractedCount: 0,
      excludedCount: 0,
      failedCount: 0,
      isComplete: true,
    },
    tokenMetrics: {
      rawTokens: 0,
      deduplicatedTokens: 0,
      finalTokens: 0,
      budgetLimit: apiJob.token_budget,
      savingsPct: 0,
      budgetRatio: 0,
    },
    artifacts: apiJob.export_formats.map((fmt) => {
      const format = fmt as "markdown" | "txt" | "json" | "html" | "jsonl" | "zip";
      return {
        format,
        byteSize: 0,
        sha256Hex: "",
        filename: `web-repo-to-ai-${apiJob.job_id}.${format === "markdown" ? "md" : format}`,
        downloadUrl: `/api/v1/jobs/${apiJob.job_id}/artifacts/${format}`,
      };
    }),
    logs: [
      {
        id: `log-${Date.now()}-backend-accepted`,
        timestamp: apiJob.created_at || new Date().toISOString(),
        level: "success",
        phase: "pending",
        message: `Conversion job confirmed by backend API (Job ID: ${apiJob.job_id}). Status: ${apiJob.status.toUpperCase()}. Target budget: ${apiJob.token_budget.toLocaleString()} tokens.`,
      },
    ],
    createdAt: apiJob.created_at || new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}
