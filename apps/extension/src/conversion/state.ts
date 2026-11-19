// Conversion State Machine for web-repo-to-ai Browser Extension

import { ExtensionApiClient } from "../api/client";
import { captureActiveTabDOM } from "../dom/extractor";
import { loadConfig } from "../storage/config";
import {
  ConversionProgress,
  ConversionResult,
  ConversionStage,
  OutputFormat,
  PageSnapshot,
  TargetModel,
} from "../types/index";

export type StateListener = (progress: ConversionProgress) => void;

export class ConversionStateMachine {
  private currentProgress: ConversionProgress = {
    stage: "idle",
    progressPct: 0,
    message: "Ready to convert",
  };
  private listeners: Set<StateListener> = new Set();
  private lastSnapshot: PageSnapshot | null = null;

  public getState(): ConversionProgress {
    return { ...this.currentProgress };
  }

  public subscribe(listener: StateListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => {
      this.listeners.delete(listener);
    };
  }

  private emit(
    stage: ConversionStage,
    progressPct: number,
    message: string,
    extra?: Partial<ConversionProgress>
  ): void {
    this.currentProgress = {
      stage,
      progressPct: Math.max(0, Math.min(100, progressPct)),
      message,
      jobId: extra?.jobId || this.currentProgress.jobId,
      error: extra?.error,
      result: extra?.result,
    };

    for (const listener of this.listeners) {
      try {
        listener(this.getState());
      } catch (err) {
        console.error("Error in conversion state listener:", err);
      }
    }
  }

  public reset(): void {
    this.emit("idle", 0, "Ready to convert");
    this.lastSnapshot = null;
  }

  public getLastSnapshot(): PageSnapshot | null {
    return this.lastSnapshot;
  }

  /**
   * Executes the full conversion pipeline using DOM capture, API client submission,
   * progress polling, and artifact retrieval.
   */
  public async executeConversion(params: {
    url: string;
    format: OutputFormat;
    tokenBudget?: number | null;
    model?: TargetModel;
    sendDomSnapshot?: boolean;
    tabId?: number;
    apiClient?: ExtensionApiClient;
  }): Promise<ConversionResult> {
    try {
      // 1. Stage: Capturing DOM if requested or available
      this.emit("capturing", 10, "Capturing page content...");
      if (params.sendDomSnapshot || params.tabId !== undefined) {
        try {
          this.lastSnapshot = await captureActiveTabDOM(params.tabId);
        } catch (domErr: any) {
          console.warn("DOM capture warning, proceeding with URL:", domErr);
        }
      }

      // 2. Resolve API client
      let client = params.apiClient;
      if (!client) {
        const config = await loadConfig();
        client = new ExtensionApiClient({
          baseUrl: config.apiUrl,
          apiKey: config.apiKey,
        });
      }

      // 3. Stage: Submitting Job
      this.emit("submitting", 20, "Submitting job to web-repo-to-ai...");

      const job = await client.createJob({
        sources: [params.url],
        token_budget: params.tokenBudget,
        target_model: params.model,
        export_formats: [params.format],
      });

      this.emit("processing", 35, "Extracting & optimizing content...", {
        jobId: job.job_id,
      });

      // 4. Stage: Polling Progress
      const completedJob = await client.pollJobUntilComplete(job.job_id, {
        intervalMs: 750,
        maxAttempts: 40,
        onProgress: (_j, pct) => {
          this.emit("processing", pct, `Processing context (${pct}%)...`, {
            jobId: job.job_id,
          });
        },
      });

      // 5. Stage: Fetching Result
      this.emit("fetching_result", 95, "Downloading converted document...", {
        jobId: completedJob.job_id,
      });

      const artifactContent = await client.fetchArtifact(completedJob.job_id, params.format);

      const durationMs = 1200; // estimated duration if not timed
      const charCount = artifactContent.length;
      const tokenCount = Math.ceil(charCount / 4);

      const result: ConversionResult = {
        jobId: completedJob.job_id,
        url: params.url,
        format: params.format,
        content: artifactContent,
        tokenCount,
        charCount,
        durationMs,
        archetype: "generic_web",
        downloadUrl: `${(client as any).baseUrl}/api/v1/jobs/${encodeURIComponent(completedJob.job_id)}/artifacts/${encodeURIComponent(params.format)}`,
      };

      // 6. Stage: Completed
      this.emit("completed", 100, "Conversion completed successfully!", {
        jobId: completedJob.job_id,
        result,
      });

      return result;
    } catch (error: any) {
      const errMsg = error.message || "Conversion failed";
      this.emit("error", 0, errMsg, { error: errMsg });
      throw error;
    }
  }
}
