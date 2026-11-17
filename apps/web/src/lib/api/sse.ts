/**
 * Real-Time Server-Sent Events (SSE) Stream Manager (`apps/web/src/lib/api/sse.ts`)
 *
 * Connects to `GET /api/v1/jobs/{job_id}/events`.
 * Decodes typed SSE events:
 * - `phase`: Pipeline phase change and progress percentage.
 * - `log`: Diagnostic log entry.
 * - `completeness`: File accounting yield metrics.
 * - `tokens`: Token budget utilization and optimization metrics.
 * - `artifact`: Artifact ready notifications.
 * - `done`: Terminal state notification.
 *
 * Implements:
 * - Automatic exponential backoff reconnection.
 * - Heartbeat timeout monitoring.
 * - Safe resource cleanup on terminal events or unmount.
 */

export interface PhaseEventData {
  phase: "pending" | "acquisition" | "normalization" | "deduplication" | "export" | "completed";
  progress_pct: number;
  message?: string;
  job_id?: string;
  timestamp?: string;
}

export interface LogEventData {
  id?: string;
  level: "info" | "warn" | "error" | "success";
  phase?: string;
  message: string;
  timestamp?: string;
}

export interface CompletenessEventData {
  discovered_count: number;
  extracted_count: number;
  excluded_count: number;
  failed_count: number;
  is_complete: boolean;
}

export interface TokensEventData {
  raw_tokens: number;
  deduplicated_tokens: number;
  final_tokens: number;
  budget_limit: number;
  savings_pct: number;
}

export interface ArtifactEventData {
  format: "markdown" | "txt" | "json" | "html" | "jsonl" | "zip";
  filename: string;
  download_url: string;
  byte_size?: number;
  sha256_hex?: string;
}

export interface DoneEventData {
  status: "completed" | "failed";
  job_id?: string;
  error?: string;
  completed_at?: string;
}

export interface JobStreamCallbacks {
  onPhase?: (data: PhaseEventData) => void;
  onLog?: (data: LogEventData) => void;
  onCompleteness?: (data: CompletenessEventData) => void;
  onTokens?: (data: TokensEventData) => void;
  onArtifact?: (data: ArtifactEventData) => void;
  onDone?: (data: DoneEventData) => void;
  onError?: (err: Error) => void;
  onConnectionChange?: (connected: boolean) => void;
}

export interface StreamOptions {
  baseUrl?: string;
  maxReconnectAttempts?: number;
  heartbeatTimeoutMs?: number;
  EventSourceClass?: typeof EventSource;
}

export class JobEventStream {
  readonly jobId: string;
  readonly streamUrl: string;
  private readonly callbacks: JobStreamCallbacks;
  private readonly maxReconnectAttempts: number;
  private readonly heartbeatTimeoutMs: number;
  private readonly EventSourceClass?: typeof EventSource;

  private eventSource: EventSource | null = null;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private heartbeatTimer: ReturnType<typeof setTimeout> | null = null;
  private isClosed = false;

  constructor(jobId: string, callbacks: JobStreamCallbacks, options?: StreamOptions) {
    this.jobId = jobId.trim();
    this.callbacks = callbacks;
    this.maxReconnectAttempts = options?.maxReconnectAttempts ?? 5;
    this.heartbeatTimeoutMs = options?.heartbeatTimeoutMs ?? 45_000;
    this.EventSourceClass =
      options?.EventSourceClass || (typeof window !== "undefined" ? window.EventSource : undefined);

    const base = options?.baseUrl || "/api/v1";
    const sanitizedBase = base.replace(/\/+$/, "");
    this.streamUrl = `${sanitizedBase}/jobs/${encodeURIComponent(this.jobId)}/events`;

    if (this.EventSourceClass) {
      this.connect();
    }
  }

  /**
   * Establishes or re-establishes the SSE connection.
   */
  private connect(): void {
    if (this.isClosed || !this.EventSourceClass) {
      return;
    }

    try {
      this.eventSource = new this.EventSourceClass(this.streamUrl);

      this.eventSource.onopen = () => {
        this.reconnectAttempts = 0;
        this.resetHeartbeat();
        this.callbacks.onConnectionChange?.(true);
      };

      this.eventSource.addEventListener("phase", (e: MessageEvent) => {
        this.resetHeartbeat();
        try {
          const data: PhaseEventData = JSON.parse(e.data);
          this.callbacks.onPhase?.(data);
        } catch {
          // ignore unparseable events
        }
      });

      this.eventSource.addEventListener("log", (e: MessageEvent) => {
        this.resetHeartbeat();
        try {
          const data: LogEventData = JSON.parse(e.data);
          this.callbacks.onLog?.(data);
        } catch {
          // ignore unparseable events
        }
      });

      this.eventSource.addEventListener("completeness", (e: MessageEvent) => {
        this.resetHeartbeat();
        try {
          const data: CompletenessEventData = JSON.parse(e.data);
          this.callbacks.onCompleteness?.(data);
        } catch {
          // ignore unparseable events
        }
      });

      this.eventSource.addEventListener("tokens", (e: MessageEvent) => {
        this.resetHeartbeat();
        try {
          const data: TokensEventData = JSON.parse(e.data);
          this.callbacks.onTokens?.(data);
        } catch {
          // ignore unparseable events
        }
      });

      this.eventSource.addEventListener("artifact", (e: MessageEvent) => {
        this.resetHeartbeat();
        try {
          const data: ArtifactEventData = JSON.parse(e.data);
          this.callbacks.onArtifact?.(data);
        } catch {
          // ignore unparseable events
        }
      });

      this.eventSource.addEventListener("done", (e: MessageEvent) => {
        this.resetHeartbeat();
        try {
          const data: DoneEventData = JSON.parse(e.data);
          this.callbacks.onDone?.(data);
        } catch {
          // ignore unparseable events
        }
        // Terminal event received: close connection cleanly
        this.close();
      });

      this.eventSource.onerror = () => {
        this.handleConnectionError();
      };
    } catch (err) {
      this.handleConnectionError();
    }
  }

  /**
   * Handles transient errors with exponential backoff reconnects.
   */
  private handleConnectionError(): void {
    if (this.isClosed) {
      return;
    }

    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }
    this.callbacks.onConnectionChange?.(false);

    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts += 1;
      const delayMs = Math.min(1000 * Math.pow(2, this.reconnectAttempts - 1), 30_000);

      this.reconnectTimer = setTimeout(() => {
        this.connect();
      }, delayMs);
    } else {
      this.callbacks.onError?.(
        new Error(
          `SSE stream failed to connect to ${this.streamUrl} after ${this.maxReconnectAttempts} attempts.`
        )
      );
      this.close();
    }
  }

  /**
   * Resets the heartbeat watchdog timer.
   */
  private resetHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearTimeout(this.heartbeatTimer);
    }

    this.heartbeatTimer = setTimeout(() => {
      // If no event was received within heartbeat period, trigger reconnect
      this.handleConnectionError();
    }, this.heartbeatTimeoutMs);
  }

  /**
   * Closes the SSE connection and frees all timers and event listeners.
   */
  close(): void {
    this.isClosed = true;

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.heartbeatTimer) {
      clearTimeout(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }

    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    this.callbacks.onConnectionChange?.(false);
  }

  /**
   * Returns whether this stream is currently closed.
   */
  get closed(): boolean {
    return this.isClosed;
  }
}

/**
 * Convenience helper to start streaming job events.
 */
export function connectJobEventStream(
  jobId: string,
  callbacks: JobStreamCallbacks,
  options?: StreamOptions
): JobEventStream {
  return new JobEventStream(jobId, callbacks, options);
}
