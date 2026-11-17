import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  JobEventStream,
  connectJobEventStream,
  type PhaseEventData,
  type LogEventData,
  type CompletenessEventData,
  type TokensEventData,
  type ArtifactEventData,
  type DoneEventData,
} from "../src/lib/api/sse";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

// Mock EventSource implementation for Vitest
class MockEventSource {
  static instances: MockEventSource[] = [];
  static autoOpen = true;
  url: string;
  onopen: ((e: any) => void) | null = null;
  onerror: ((e: any) => void) | null = null;
  listeners: Record<string, ((e: any) => void)[]> = {};
  closed = false;

  constructor(url: string) {
    this.url = url;
    MockEventSource.instances.push(this);
    // Simulate async open
    setTimeout(() => {
      if (!this.closed && this.onopen && MockEventSource.autoOpen) {
        this.onopen({ type: "open" });
      }
    }, 1);
  }

  addEventListener(type: string, listener: (e: any) => void) {
    if (!this.listeners[type]) {
      this.listeners[type] = [];
    }
    this.listeners[type].push(listener);
  }

  removeEventListener(type: string, listener: (e: any) => void) {
    if (this.listeners[type]) {
      this.listeners[type] = this.listeners[type].filter((l) => l !== listener);
    }
  }

  dispatchEvent(type: string, data: any) {
    const event = {
      type,
      data: typeof data === "string" ? data : JSON.stringify(data),
    };
    if (this.listeners[type]) {
      for (const listener of this.listeners[type]) {
        listener(event);
      }
    }
  }

  close() {
    this.closed = true;
  }
}

describe("JobEventStream SSE Client Lifecycle & Events", () => {
  beforeEach(() => {
    MockEventSource.instances = [];
    MockEventSource.autoOpen = true;
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("constructs stream URL targeting /api/v1/jobs/{job_id}/events", () => {
    const stream = connectJobEventStream(
      "019335c4-7c20-7000-8000-000000000001",
      {},
      { EventSourceClass: MockEventSource as any }
    );

    expect(stream.streamUrl).toBe("/api/v1/jobs/019335c4-7c20-7000-8000-000000000001/events");
    expect(MockEventSource.instances).toHaveLength(1);
    expect(MockEventSource.instances[0].url).toBe(stream.streamUrl);
    stream.close();
  });

  it("triggers onConnectionChange(true) when EventSource opens", () => {
    const onConnectionChange = vi.fn();
    const stream = connectJobEventStream(
      "job-101",
      { onConnectionChange },
      { EventSourceClass: MockEventSource as any }
    );

    vi.advanceTimersByTime(2);
    expect(onConnectionChange).toHaveBeenCalledWith(true);
    stream.close();
  });

  it("decodes and dispatches phase events", () => {
    const onPhase = vi.fn();
    const stream = connectJobEventStream(
      "job-102",
      { onPhase },
      { EventSourceClass: MockEventSource as any }
    );

    vi.advanceTimersByTime(2);
    const mockSource = MockEventSource.instances[0];

    const phasePayload: PhaseEventData = {
      phase: "normalization",
      progress_pct: 45,
      message: "Generating CDM canonical representation",
    };
    mockSource.dispatchEvent("phase", phasePayload);

    expect(onPhase).toHaveBeenCalledWith(phasePayload);
    stream.close();
  });

  it("decodes and dispatches log events", () => {
    const onLog = vi.fn();
    const stream = connectJobEventStream(
      "job-103",
      { onLog },
      { EventSourceClass: MockEventSource as any }
    );

    vi.advanceTimersByTime(2);
    const mockSource = MockEventSource.instances[0];

    const logPayload: LogEventData = {
      id: "log-1",
      level: "info",
      phase: "acquisition",
      message: "Fetched HTML payload (14.2 KB)",
    };
    mockSource.dispatchEvent("log", logPayload);

    expect(onLog).toHaveBeenCalledWith(logPayload);
    stream.close();
  });

  it("decodes and dispatches completeness accounting events", () => {
    const onCompleteness = vi.fn();
    const stream = connectJobEventStream(
      "job-104",
      { onCompleteness },
      { EventSourceClass: MockEventSource as any }
    );

    vi.advanceTimersByTime(2);
    const mockSource = MockEventSource.instances[0];

    const completenessPayload: CompletenessEventData = {
      discovered_count: 10,
      extracted_count: 8,
      excluded_count: 2,
      failed_count: 0,
      is_complete: true,
    };
    mockSource.dispatchEvent("completeness", completenessPayload);

    expect(onCompleteness).toHaveBeenCalledWith(completenessPayload);
    stream.close();
  });

  it("decodes and dispatches token optimization metrics events", () => {
    const onTokens = vi.fn();
    const stream = connectJobEventStream(
      "job-105",
      { onTokens },
      { EventSourceClass: MockEventSource as any }
    );

    vi.advanceTimersByTime(2);
    const mockSource = MockEventSource.instances[0];

    const tokensPayload: TokensEventData = {
      raw_tokens: 50000,
      deduplicated_tokens: 38000,
      final_tokens: 35000,
      budget_limit: 160000,
      savings_pct: 30.0,
    };
    mockSource.dispatchEvent("tokens", tokensPayload);

    expect(onTokens).toHaveBeenCalledWith(tokensPayload);
    stream.close();
  });

  it("decodes artifact ready events", () => {
    const onArtifact = vi.fn();
    const stream = connectJobEventStream(
      "job-106",
      { onArtifact },
      { EventSourceClass: MockEventSource as any }
    );

    vi.advanceTimersByTime(2);
    const mockSource = MockEventSource.instances[0];

    const artifactPayload: ArtifactEventData = {
      format: "markdown",
      filename: "web-repo-to-ai-job-106.md",
      download_url: "/api/v1/jobs/job-106/artifacts/markdown",
      byte_size: 12040,
      sha256_hex: "abcdef1234567890",
    };
    mockSource.dispatchEvent("artifact", artifactPayload);

    expect(onArtifact).toHaveBeenCalledWith(artifactPayload);
    stream.close();
  });

  it("dispatches done event and automatically closes connection to prevent SSE leaks", () => {
    const onDone = vi.fn();
    const onConnectionChange = vi.fn();
    const stream = connectJobEventStream(
      "job-107",
      { onDone, onConnectionChange },
      { EventSourceClass: MockEventSource as any }
    );

    vi.advanceTimersByTime(2);
    const mockSource = MockEventSource.instances[0];

    const donePayload: DoneEventData = {
      status: "completed",
      job_id: "job-107",
    };
    mockSource.dispatchEvent("done", donePayload);

    expect(onDone).toHaveBeenCalledWith(donePayload);
    expect(stream.closed).toBe(true);
    expect(mockSource.closed).toBe(true);
    expect(onConnectionChange).toHaveBeenLastCalledWith(false);
  });

  it("attempts exponential backoff reconnect on connection error", () => {
    const onConnectionChange = vi.fn();
    const stream = connectJobEventStream(
      "job-108",
      { onConnectionChange },
      { EventSourceClass: MockEventSource as any, maxReconnectAttempts: 3 }
    );

    vi.advanceTimersByTime(2);
    expect(MockEventSource.instances).toHaveLength(1);

    // Trigger error on first instance
    MockEventSource.instances[0].onerror?.({ type: "error" });
    expect(MockEventSource.instances[0].closed).toBe(true);
    expect(onConnectionChange).toHaveBeenLastCalledWith(false);

    // Advance by initial reconnect delay (1000ms)
    vi.advanceTimersByTime(1005);
    expect(MockEventSource.instances).toHaveLength(2);

    stream.close();
  });

  it("invokes onError when maximum reconnect attempts are exceeded", () => {
    MockEventSource.autoOpen = false;
    const onError = vi.fn();
    const stream = connectJobEventStream(
      "job-109",
      { onError },
      { EventSourceClass: MockEventSource as any, maxReconnectAttempts: 2 }
    );

    // Fail 1
    MockEventSource.instances[0].onerror?.({ type: "error" });
    vi.advanceTimersByTime(1005);
    expect(MockEventSource.instances).toHaveLength(2);

    // Fail 2
    MockEventSource.instances[1].onerror?.({ type: "error" });
    vi.advanceTimersByTime(2005);
    expect(MockEventSource.instances).toHaveLength(3);

    // Fail 3 (exceeds max 2 attempts)
    MockEventSource.instances[2].onerror?.({ type: "error" });

    expect(onError).toHaveBeenCalled();
    expect(stream.closed).toBe(true);
  });
});

describe("Workspace Route SSE Integration Contract", () => {
  it("workspace/+page.svelte binds connectJobEventStream with all event callbacks and onDestroy cleanup", () => {
    const pagePath = resolve(process.cwd(), "src/routes/workspace/+page.svelte");
    const content = readFileSync(pagePath, "utf-8");

    expect(content).toContain("connectJobEventStream");
    expect(content).toContain("onDestroy");
    expect(content).toContain("stream.close()");
    expect(content).toContain("onPhase:");
    expect(content).toContain("onLog:");
    expect(content).toContain("onCompleteness:");
    expect(content).toContain("onTokens:");
    expect(content).toContain("onArtifact:");
    expect(content).toContain("onDone:");
  });
});
