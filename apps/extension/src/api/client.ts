// REST API Client for web-repo-to-ai Browser Extension

import { ConversionResult, DetectedArchetype, OutputFormat, TargetModel } from "../types/index";

export interface ApiClientConfig {
  baseUrl: string;
  apiKey?: string;
  fetchFn?: typeof fetch;
}

export interface CreateJobParams {
  sources: string[];
  token_budget?: number | null;
  target_model?: TargetModel;
  export_formats?: OutputFormat[];
  deduplicate_exact?: boolean;
}

export interface ApiJobResponse {
  job_id: string;
  status: "queued" | "in_progress" | "completed" | "failed" | "cancelled";
  sources: string[];
  token_budget: number;
  target_model: string;
  export_formats: string[];
  created_at: string;
  error_message?: string;
  details?: Record<string, unknown>;
}

export class ExtensionApiClient {
  private readonly baseUrl: string;
  private readonly apiKey: string;
  private readonly fetchFn: typeof fetch;

  constructor(config: ApiClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/+$/, "");
    this.apiKey = config.apiKey ? config.apiKey.trim() : "";
    this.fetchFn =
      config.fetchFn ||
      (typeof window !== "undefined" ? window.fetch.bind(window) : globalThis.fetch);
  }

  private getHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      Accept: "application/json, application/problem+json, text/plain",
      "Content-Type": "application/json",
    };
    if (this.apiKey) {
      headers["Authorization"] = `Bearer ${this.apiKey}`;
    }
    return headers;
  }

  /**
   * Submits a new conversion job to the web-repo-to-ai REST API.
   */
  async createJob(params: CreateJobParams): Promise<ApiJobResponse> {
    const endpoint = `${this.baseUrl}/api/v1/jobs`;
    const payload: Record<string, unknown> = {
      sources: params.sources,
    };

    if (params.token_budget !== undefined && params.token_budget !== null) {
      payload.token_budget = params.token_budget;
    }
    if (params.target_model) {
      payload.target_model = params.target_model;
    }
    if (params.export_formats && params.export_formats.length > 0) {
      payload.export_formats = params.export_formats;
    }
    if (params.deduplicate_exact !== undefined) {
      payload.deduplicate_exact = params.deduplicate_exact;
    }

    const response = await this.fetchFn(endpoint, {
      method: "POST",
      headers: this.getHeaders(),
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      let errDetail = `HTTP ${response.status}: ${response.statusText}`;
      try {
        const problem = await response.json();
        if (problem.detail) {
          errDetail = problem.detail;
        } else if (problem.title) {
          errDetail = problem.title;
        }
      } catch {
        // Use default error string if not JSON
      }
      throw new Error(`Failed to create conversion job: ${errDetail}`);
    }

    return (await response.json()) as ApiJobResponse;
  }

  /**
   * Inspects the current state of a conversion job.
   */
  async getJobStatus(jobId: string): Promise<ApiJobResponse> {
    const endpoint = `${this.baseUrl}/api/v1/jobs/${encodeURIComponent(jobId)}`;
    const response = await this.fetchFn(endpoint, {
      method: "GET",
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Failed to inspect job ${jobId}: HTTP ${response.status}`);
    }

    return (await response.json()) as ApiJobResponse;
  }

  /**
   * Downloads a generated artifact for a completed job.
   */
  async fetchArtifact(jobId: string, format: OutputFormat): Promise<string> {
    const endpoint = `${this.baseUrl}/api/v1/jobs/${encodeURIComponent(jobId)}/artifacts/${encodeURIComponent(format)}`;
    const response = await this.fetchFn(endpoint, {
      method: "GET",
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(
        `Failed to fetch artifact for job ${jobId} (${format}): HTTP ${response.status}`
      );
    }

    return await response.text();
  }

  /**
   * Polls job status until it terminates (completed, failed, cancelled) or reaches max timeout.
   */
  async pollJobUntilComplete(
    jobId: string,
    options?: {
      maxAttempts?: number;
      intervalMs?: number;
      onProgress?: (job: ApiJobResponse, progressPct: number) => void;
    }
  ): Promise<ApiJobResponse> {
    const maxAttempts = options?.maxAttempts || 30;
    const intervalMs = options?.intervalMs || 1000;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      const job = await this.getJobStatus(jobId);

      // Estimated progress based on status and attempts
      let progressPct = 30 + Math.min(60, Math.floor((attempt / maxAttempts) * 60));
      if (job.status === "completed") {
        progressPct = 100;
      }

      if (options?.onProgress) {
        options.onProgress(job, progressPct);
      }

      if (job.status === "completed") {
        return job;
      }
      if (job.status === "failed") {
        throw new Error(job.error_message || "Conversion job processing failed on server");
      }
      if (job.status === "cancelled") {
        throw new Error("Conversion job was cancelled");
      }

      if (attempt < maxAttempts) {
        await new Promise((resolve) => setTimeout(resolve, intervalMs));
      }
    }

    throw new Error(`Conversion job ${jobId} timed out after ${maxAttempts * intervalMs}ms`);
  }

  /**
   * Orchestrates the complete conversion flow: submits job, polls progress, and downloads result artifact.
   */
  async executeFullConversion(params: {
    url: string;
    format: OutputFormat;
    tokenBudget?: number | null;
    model?: TargetModel;
    archetype?: DetectedArchetype;
    onProgress?: (phase: string, progressPct: number) => void;
  }): Promise<ConversionResult> {
    const startTime = Date.now();

    // 1. Submit Job
    if (params.onProgress) {
      params.onProgress("Submitting conversion job to backend...", 15);
    }

    const job = await this.createJob({
      sources: [params.url],
      token_budget: params.tokenBudget,
      target_model: params.model,
      export_formats: [params.format],
    });

    // 2. Poll for completion
    if (params.onProgress) {
      params.onProgress("Processing & optimizing content...", 40);
    }

    const completedJob = await this.pollJobUntilComplete(job.job_id, {
      intervalMs: 800,
      maxAttempts: 35,
      onProgress: (_j, pct) => {
        if (params.onProgress) {
          params.onProgress("Processing & optimizing content...", pct);
        }
      },
    });

    // 3. Fetch Artifact
    if (params.onProgress) {
      params.onProgress("Downloading converted document...", 92);
    }

    const artifactContent = await this.fetchArtifact(completedJob.job_id, params.format);
    const durationMs = Date.now() - startTime;
    const charCount = artifactContent.length;
    const tokenCount = Math.ceil(charCount / 4);

    const downloadUrl = `${this.baseUrl}/api/v1/jobs/${encodeURIComponent(completedJob.job_id)}/artifacts/${encodeURIComponent(params.format)}`;

    return {
      jobId: completedJob.job_id,
      url: params.url,
      format: params.format,
      content: artifactContent,
      tokenCount,
      charCount,
      durationMs,
      archetype: params.archetype || "generic_web",
      downloadUrl,
    };
  }
}
