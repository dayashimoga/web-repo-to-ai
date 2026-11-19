import { beforeEach, describe, expect, it, vi } from "vitest";
import { ExtensionApiClient } from "../src/api/client";
import { ConversionStateMachine } from "../src/conversion/state";
import { ConversionProgress } from "../src/types/index";

describe("Conversion State Machine (apps/extension)", () => {
  let stateMachine: ConversionStateMachine;

  beforeEach(() => {
    stateMachine = new ConversionStateMachine();
  });

  it("initializes with idle stage", () => {
    const state = stateMachine.getState();
    expect(state.stage).toBe("idle");
    expect(state.progressPct).toBe(0);
    expect(state.message).toMatch(/ready/i);
  });

  it("notifies subscribers on stage transitions", async () => {
    const states: ConversionProgress[] = [];
    const unsubscribe = stateMachine.subscribe((progress) => {
      states.push(progress);
    });

    // Mock API client
    const mockApiClient = {
      createJob: vi.fn().mockResolvedValue({ job_id: "test-job-123", status: "queued" }),
      pollJobUntilComplete: vi
        .fn()
        .mockResolvedValue({ job_id: "test-job-123", status: "completed" }),
      fetchArtifact: vi.fn().mockResolvedValue("# Test Article Content"),
      baseUrl: "http://localhost:3000",
    } as unknown as ExtensionApiClient;

    const result = await stateMachine.executeConversion({
      url: "https://example.com/article",
      format: "markdown",
      tokenBudget: 16000,
      apiClient: mockApiClient,
    });

    expect(result.jobId).toBe("test-job-123");
    expect(result.content).toBe("# Test Article Content");

    const stageSequence = states.map((s) => s.stage);
    expect(stageSequence).toContain("capturing");
    expect(stageSequence).toContain("submitting");
    expect(stageSequence).toContain("processing");
    expect(stageSequence).toContain("fetching_result");
    expect(stageSequence).toContain("completed");

    unsubscribe();
  });

  it("transitions to error stage on failure and throws", async () => {
    const states: ConversionProgress[] = [];
    stateMachine.subscribe((progress) => states.push(progress));

    const mockApiClient = {
      createJob: vi.fn().mockRejectedValue(new Error("Network connection refused")),
    } as unknown as ExtensionApiClient;

    await expect(
      stateMachine.executeConversion({
        url: "https://example.com/unreachable",
        format: "markdown",
        apiClient: mockApiClient,
      })
    ).rejects.toThrow(/Network connection refused/);

    const finalState = stateMachine.getState();
    expect(finalState.stage).toBe("error");
    expect(finalState.error).toMatch(/Network connection refused/);
  });

  it("resets back to idle state", () => {
    stateMachine.reset();
    const state = stateMachine.getState();
    expect(state.stage).toBe("idle");
    expect(state.progressPct).toBe(0);
    expect(state.error).toBeUndefined();
    expect(state.result).toBeUndefined();
  });
});
