import { beforeEach, describe, expect, it, vi } from "vitest";
import { ApiJobResponse, ExtensionApiClient } from "../src/api/client";

describe("Extension REST API Client (apps/extension)", () => {
  let mockFetch: ReturnType<typeof vi.fn>;
  let client: ExtensionApiClient;

  beforeEach(() => {
    mockFetch = vi.fn();
    client = new ExtensionApiClient({
      baseUrl: "http://localhost:3000",
      apiKey: "wr_live_secret123",
      fetchFn: mockFetch as any,
    });
  });

  describe("createJob", () => {
    it("submits POST /api/v1/jobs with headers and body", async () => {
      const mockJob: ApiJobResponse = {
        job_id: "0193425b-7b24-7000-8000-000000000001",
        status: "queued",
        sources: ["https://github.com/rust-lang/rust"],
        token_budget: 32000,
        target_model: "gpt-4o",
        export_formats: ["markdown"],
        created_at: new Date().toISOString(),
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 202,
        json: async () => mockJob,
      });

      const result = await client.createJob({
        sources: ["https://github.com/rust-lang/rust"],
        token_budget: 32000,
        target_model: "gpt-4o",
        export_formats: ["markdown"],
      });

      expect(result.job_id).toBe(mockJob.job_id);
      expect(result.status).toBe("queued");
      expect(mockFetch).toHaveBeenCalledWith(
        "http://localhost:3000/api/v1/jobs",
        expect.objectContaining({
          method: "POST",
          headers: expect.objectContaining({
            Authorization: "Bearer wr_live_secret123",
            "Content-Type": "application/json",
          }),
        })
      );
    });

    it("throws error with problem details on 400 Bad Request", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 400,
        statusText: "Bad Request",
        json: async () => ({
          type: "https://example.com/probs/validation",
          title: "Validation Error",
          detail: "URL contains disallowed private IP",
        }),
      });

      await expect(client.createJob({ sources: ["http://127.0.0.1:80"] })).rejects.toThrow(
        /URL contains disallowed private IP/i
      );
    });
  });

  describe("getJobStatus & fetchArtifact", () => {
    it("fetches job status via GET /api/v1/jobs/{job_id}", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          job_id: "job-123",
          status: "completed",
        }),
      });

      const res = await client.getJobStatus("job-123");
      expect(res.job_id).toBe("job-123");
      expect(res.status).toBe("completed");
    });

    it("downloads artifact content as text", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        text: async () => "# Converted Title\n\nContent body here.",
      });

      const content = await client.fetchArtifact("job-123", "markdown");
      expect(content).toContain("# Converted Title");
    });
  });

  describe("pollJobUntilComplete", () => {
    it("polls until status is completed", async () => {
      // First attempt: in_progress
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ job_id: "job-1", status: "in_progress" }),
      });
      // Second attempt: completed
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ job_id: "job-1", status: "completed" }),
      });

      const progressCalls: number[] = [];
      const completed = await client.pollJobUntilComplete("job-1", {
        intervalMs: 10,
        maxAttempts: 5,
        onProgress: (_j, pct) => progressCalls.push(pct),
      });

      expect(completed.status).toBe("completed");
      expect(progressCalls.length).toBeGreaterThanOrEqual(2);
      expect(progressCalls[progressCalls.length - 1]).toBe(100);
    });

    it("throws error when job fails", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          job_id: "job-err",
          status: "failed",
          error_message: "SSRF host rejected",
        }),
      });

      await expect(
        client.pollJobUntilComplete("job-err", { intervalMs: 10, maxAttempts: 3 })
      ).rejects.toThrow(/SSRF host rejected/i);
    });
  });

  describe("executeFullConversion", () => {
    it("runs complete lifecycle from createJob to fetchArtifact", async () => {
      // 1. Create Job -> 202 Accepted
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 202,
        json: async () => ({
          job_id: "job-full",
          status: "queued",
          sources: ["https://example.com"],
        }),
      });

      // 2. Poll Status -> completed
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          job_id: "job-full",
          status: "completed",
        }),
      });

      // 3. Fetch Artifact -> Markdown text
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        text: async () => "# Converted Document\n\nFull text.",
      });

      const progressPhases: string[] = [];
      const result = await client.executeFullConversion({
        url: "https://example.com",
        format: "markdown",
        tokenBudget: 32000,
        archetype: "generic_web",
        onProgress: (phase) => progressPhases.push(phase),
      });

      expect(result.jobId).toBe("job-full");
      expect(result.content).toContain("# Converted Document");
      expect(result.format).toBe("markdown");
      expect(result.charCount).toBeGreaterThan(0);
      expect(result.tokenCount).toBeGreaterThan(0);
      expect(progressPhases.length).toBeGreaterThan(0);
    });
  });
});
