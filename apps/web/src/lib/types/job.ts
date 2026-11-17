/**
 * Conversion Job Domain Model & State Machine
 *
 * Implements:
 * 1. REST / Streaming API alignment matching docs/API.md & docs/DATA_MODEL.md.
 * 2. 4-Stage Pipeline Lifecycle: Acquisition -> Normalization -> Deduplication -> Export.
 * 3. File Completeness Accounting Invariant:
 *    discovered_count = extracted_count + excluded_count + failed_count
 * 4. Token metrics and structured windowed logging.
 */

export type JobStatus =
  "queued" | "leased" | "running" | "completed" | "partial" | "failed" | "cancelled";

export type PipelinePhase =
  "pending" | "acquisition" | "normalization" | "deduplication" | "export" | "completed";

export type StageState = "pending" | "in_progress" | "completed" | "failed" | "skipped";

export interface PipelineStageInfo {
  id: PipelinePhase;
  name: string;
  description: string;
  state: StageState;
  progressPct: number;
  startedAt?: string;
  completedAt?: string;
  detail?: string;
}

export interface CompletenessReport {
  discoveredCount: number;
  extractedCount: number;
  excludedCount: number;
  failedCount: number;
  isComplete: boolean;
}

export interface JobTokenMetrics {
  rawTokens: number;
  deduplicatedTokens: number;
  finalTokens: number;
  budgetLimit: number;
  savingsPct: number;
  budgetRatio: number;
}

export interface JobLogEntry {
  id: string;
  timestamp: string;
  level: "info" | "warn" | "error" | "success";
  phase: PipelinePhase;
  message: string;
}

export interface JobArtifactItem {
  format: "markdown" | "txt" | "json" | "html" | "jsonl" | "zip";
  byteSize: number;
  sha256Hex: string;
  filename: string;
  downloadUrl: string;
}

export interface ConversionJob {
  id: string;
  sourceUrl: string;
  detectedSource: string;
  targetFormat: "markdown" | "json" | "xml" | "txt" | "jsonl" | "zip";
  modelPreset: string;
  status: JobStatus;
  currentPhase: PipelinePhase;
  overallProgress: number;
  stages: PipelineStageInfo[];
  completeness: CompletenessReport;
  tokenMetrics: JobTokenMetrics;
  artifacts: JobArtifactItem[];
  logs: JobLogEntry[];
  error?: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
}

/**
 * Creates standard 4-stage pipeline definition.
 */
export function createDefaultPipelineStages(): PipelineStageInfo[] {
  return [
    {
      id: "acquisition",
      name: "Acquisition & SSRF Defense",
      description: "Zero-trust network validation, DNS re-check, and raw content extraction.",
      state: "pending",
      progressPct: 0,
    },
    {
      id: "normalization",
      name: "CDM Canonicalization",
      description: "AST parsing, boilerplate removal, and schema canonicalization.",
      state: "pending",
      progressPct: 0,
    },
    {
      id: "deduplication",
      name: "MinHash Deduplication",
      description: "Exact SHA-256 and MinHash LSH near-duplicate section elimination.",
      state: "pending",
      progressPct: 0,
    },
    {
      id: "export",
      name: "Export & Token Fit",
      description: "Target format generation, BPE token budgeting, and checksum validation.",
      state: "pending",
      progressPct: 0,
    },
  ];
}

/**
 * Factory creating a fresh ConversionJob initialized with verifiable initial state.
 */
export function createInitialJob(
  sourceUrl: string,
  options?: {
    id?: string;
    detectedSource?: string;
    targetFormat?: "markdown" | "json" | "xml" | "txt" | "jsonl" | "zip";
    modelPreset?: string;
    budgetLimit?: number;
  }
): ConversionJob {
  const now = new Date().toISOString();
  const id = options?.id || `job-${Math.random().toString(36).substring(2, 10)}`;
  const budgetLimit = options?.budgetLimit || 160000;

  return {
    id,
    sourceUrl,
    detectedSource: options?.detectedSource || "Auto-Detected Source",
    targetFormat: options?.targetFormat || "markdown",
    modelPreset: options?.modelPreset || "claude-3.5-sonnet",
    status: "queued",
    currentPhase: "pending",
    overallProgress: 0,
    stages: createDefaultPipelineStages(),
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
      budgetLimit,
      savingsPct: 0,
      budgetRatio: 0,
    },
    artifacts: [],
    logs: [
      {
        id: `log-${Date.now()}-0`,
        timestamp: now,
        level: "info",
        phase: "pending",
        message: `Job initialized for target URL: ${sourceUrl}`,
      },
    ],
    createdAt: now,
    updatedAt: now,
  };
}

/**
 * Validates the completeness invariant:
 * discovered = extracted + excluded + failed
 */
export function verifyCompletenessInvariant(report: CompletenessReport): boolean {
  return (
    report.discoveredCount === report.extractedCount + report.excludedCount + report.failedCount
  );
}

/**
 * Calculates overall weighted progress percentage across all active pipeline stages.
 */
export function calculateOverallProgress(stages: PipelineStageInfo[]): number {
  if (stages.length === 0) return 0;
  const total = stages.reduce((acc, stage) => {
    if (stage.state === "completed") return acc + 100;
    if (stage.state === "in_progress") return acc + stage.progressPct;
    return acc;
  }, 0);
  return Math.min(100, Math.round(total / stages.length));
}
