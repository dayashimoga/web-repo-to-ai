import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  ApiClient,
  ApiClientError,
  defaultApiClient,
  mapApiJobToConversionJob,
  mapApiStatusToJobStatus,
  mapStatusToPhase,
  type ApiJobResponse,
  type ApiProblemDetails,
  type CreateJobPayload,
  type DetectSourcesResponse,
} from "../src/lib/api/index";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("ApiClient REST Integration & Transport", () => {
  const originalFetch = globalThis.fetch;

  afterEach(() => {
    globalThis.fetch = originalFetch;
    vi.restoreAllMocks();
  });

  it("initializes with default relative base URL /api/v1 and headers", () => {
    const client = new ApiClient();
    expect(client).toBeDefined();
    expect(defaultApiClient).toBeDefined();
    expect(client.getArtifactUrl("test-uuid-1", "markdown")).toBe(
      "/api/v1/jobs/test-uuid-1/artifacts/markdown"
    );
    expect(client.getStreamUrl("test-uuid-1")).toBe("/api/v1/jobs/test-uuid-1/stream");
  });

  it("honors custom base URL and sanitizes trailing slashes", () => {
    const client = new ApiClient({ baseUrl: "http://localhost:3000/api/v1/" });
    expect(client.getArtifactUrl("job-abc", "zip")).toBe(
      "http://localhost:3000/api/v1/jobs/job-abc/artifacts/zip"
    );
    expect(client.getStreamUrl("job-abc")).toBe("http://localhost:3000/api/v1/jobs/job-abc/stream");
  });

  it("dispatches createJob with correct POST headers and parses 202 response", async () => {
    const mockResponse: ApiJobResponse = {
      job_id: "018f2345-6789-7000-8000-000000000001",
      status: "queued",
      sources: ["https://github.com/org/repo"],
      token_budget: 160000,
      target_model: "claude-3-5-sonnet",
      export_formats: ["markdown", "zip"],
      created_at: "2026-11-17T20:00:00Z",
    };

    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 202,
      statusText: "Accepted",
      headers: new Headers({ "content-type": "application/json" }),
      json: async () => mockResponse,
    });

    const client = new ApiClient({ fetchFn: mockFetch });
    const payload: CreateJobPayload = {
      sources: ["https://github.com/org/repo"],
      token_budget: 160000,
      target_model: "claude-3-5-sonnet",
      export_formats: ["markdown", "zip"],
    };

    const result = await client.createJob(payload);

    expect(mockFetch).toHaveBeenCalledTimes(1);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe("/api/v1/jobs");
    expect(init.method).toBe("POST");
    expect(init.headers["Content-Type"]).toBe("application/json");
    expect(init.headers["Accept"]).toContain("application/json");
    expect(JSON.parse(init.body)).toEqual(payload);
    expect(result).toEqual(mockResponse);
    expect(result.job_id).toBe("018f2345-6789-7000-8000-000000000001");
    expect(result.status).toBe("queued");
  });

  it("dispatches getJob with GET and decodes retrieved job state", async () => {
    const mockResponse: ApiJobResponse = {
      job_id: "018f2345-6789-7000-8000-000000000002",
      status: "completed",
      sources: ["https://example.com/docs"],
      token_budget: 32000,
      target_model: "gpt-4o",
      export_formats: ["json"],
      created_at: "2026-11-17T19:50:00Z",
    };

    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      headers: new Headers({ "content-type": "application/json" }),
      json: async () => mockResponse,
    });

    const client = new ApiClient({ fetchFn: mockFetch });
    const result = await client.getJob("018f2345-6789-7000-8000-000000000002");

    expect(mockFetch).toHaveBeenCalledWith(
      "/api/v1/jobs/018f2345-6789-7000-8000-000000000002",
      expect.objectContaining({ method: "GET" })
    );
    expect(result.status).toBe("completed");
  });

  it("dispatches detectSources with POST and decodes classified URLs", async () => {
    const mockResponse: DetectSourcesResponse = {
      detected: [
        {
          url: "https://github.com/org/repo",
          classification: "public_github",
          normalized_url: "https://github.com/org/repo",
          repo_owner: "org",
          repo_name: "repo",
          git_ref: "main",
        },
      ],
      total_detected: 1,
    };

    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      headers: new Headers({ "content-type": "application/json" }),
      json: async () => mockResponse,
    });

    const client = new ApiClient({ fetchFn: mockFetch });
    const result = await client.detectSources(["https://github.com/org/repo"]);

    expect(mockFetch).toHaveBeenCalledWith(
      "/api/v1/sources/detect",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ urls: ["https://github.com/org/repo"] }),
      })
    );
    expect(result.detected).toHaveLength(1);
    expect(result.detected[0].classification).toBe("public_github");
  });

  it("correctly decodes RFC 7807 Problem Details on HTTP 400", async () => {
    const problemDetails: ApiProblemDetails = {
      type: "https://errors.web-repo-to-ai.org/validation-error",
      title: "Validation Error",
      status: 400,
      detail: "Invalid token budget: must be between 1,000 and 2,000,000",
      request_id: "req-uuid-400",
      invalid_params: [{ name: "token_budget", reason: "Value 50 is below minimum 1000" }],
    };

    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 400,
      statusText: "Bad Request",
      headers: new Headers({ "content-type": "application/problem+json" }),
      json: async () => problemDetails,
    });

    const client = new ApiClient({ fetchFn: mockFetch });

    await expect(
      client.createJob({ sources: ["https://example.com"], token_budget: 50 })
    ).rejects.toThrowError(ApiClientError);

    try {
      await client.createJob({ sources: ["https://example.com"], token_budget: 50 });
    } catch (err) {
      expect(err).toBeInstanceOf(ApiClientError);
      const apiErr = err as ApiClientError;
      expect(apiErr.status).toBe(400);
      expect(apiErr.message).toBe("Invalid token budget: must be between 1,000 and 2,000,000");
      expect(apiErr.requestId).toBe("req-uuid-400");
      expect(apiErr.problem?.invalid_params).toHaveLength(1);
    }
  });

  it("handles transport and network errors cleanly", async () => {
    const mockFetch = vi.fn().mockRejectedValue(new Error("Connection refused"));
    const client = new ApiClient({ fetchFn: mockFetch });

    await expect(client.getJob("job-123")).rejects.toThrowError(ApiClientError);

    try {
      await client.getJob("job-123");
    } catch (err) {
      expect(err).toBeInstanceOf(ApiClientError);
      const apiErr = err as ApiClientError;
      expect(apiErr.status).toBe(0);
      expect(apiErr.message).toContain("Failed to connect to conversion API service");
    }
  });
});

describe("API Data Adapter (mapApiJobToConversionJob)", () => {
  it("maps queued ApiJobResponse into valid ConversionJob domain model", () => {
    const apiJob: ApiJobResponse = {
      job_id: "018f2345-6789-7000-8000-000000000099",
      status: "queued",
      sources: ["https://github.com/example/repo"],
      token_budget: 160000,
      target_model: "claude-3-5-sonnet",
      export_formats: ["markdown", "zip"],
      created_at: "2026-11-17T20:10:00Z",
    };

    const domainJob = mapApiJobToConversionJob(apiJob, {
      detectedSource: "GitHub Public Repository",
      targetFormat: "markdown",
    });

    expect(domainJob.id).toBe("018f2345-6789-7000-8000-000000000099");
    expect(domainJob.sourceUrl).toBe("https://github.com/example/repo");
    expect(domainJob.detectedSource).toBe("GitHub Public Repository");
    expect(domainJob.status).toBe("queued");
    expect(domainJob.currentPhase).toBe("pending");
    expect(domainJob.overallProgress).toBe(0);
    expect(domainJob.tokenMetrics.budgetLimit).toBe(160000);
    expect(domainJob.stages).toHaveLength(4);
    expect(domainJob.artifacts).toHaveLength(2);
    expect(domainJob.artifacts[0].downloadUrl).toBe(
      "/api/v1/jobs/018f2345-6789-7000-8000-000000000099/artifacts/markdown"
    );
    expect(domainJob.logs[0].message).toContain("Conversion job confirmed by backend API");
  });

  it("maps processing and completed ApiJobResponse states correctly", () => {
    expect(mapApiStatusToJobStatus("queued")).toBe("queued");
    expect(mapApiStatusToJobStatus("processing")).toBe("running");
    expect(mapApiStatusToJobStatus("completed")).toBe("completed");
    expect(mapApiStatusToJobStatus("failed")).toBe("failed");
    expect(mapApiStatusToJobStatus("cancelled")).toBe("cancelled");

    expect(mapStatusToPhase("queued")).toBe("pending");
    expect(mapStatusToPhase("running")).toBe("acquisition");
    expect(mapStatusToPhase("completed")).toBe("completed");

    const runningApiJob: ApiJobResponse = {
      job_id: "018f2345-6789-7000-8000-000000000100",
      status: "processing",
      sources: ["https://example.com"],
      token_budget: 64000,
      target_model: "gpt-4o",
      export_formats: ["json"],
      created_at: "2026-11-17T20:12:00Z",
    };

    const runningJob = mapApiJobToConversionJob(runningApiJob);
    expect(runningJob.status).toBe("running");
    expect(runningJob.currentPhase).toBe("acquisition");
    expect(runningJob.stages[0].state).toBe("in_progress");
  });

  it("dispatches getJobPreview with query parameters and decodes preview payload", async () => {
    const mockPreview = {
      job_id: "018f2345-6789-7000-8000-000000000099",
      format: "markdown",
      content: "# Live Preview\n```rust\nfn main() {}\n```",
      total_lines: 3,
      total_tokens: 12,
      total_bytes: 45,
      truncated: false,
      max_lines: 500,
      files: [
        {
          id: "sec-1",
          path: "src/main.rs",
          language: "rust",
          byte_size: 45,
          token_count: 12,
          line_count: 3,
        },
      ],
    };

    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      headers: new Headers({ "content-type": "application/json" }),
      json: async () => mockPreview,
    });

    const client = new ApiClient({ fetchFn: mockFetch });
    const result = await client.getJobPreview("018f2345-6789-7000-8000-000000000099", {
      format: "markdown",
      maxLines: 500,
      filePath: "src/main.rs",
    });

    expect(mockFetch).toHaveBeenCalledWith(
      "/api/v1/jobs/018f2345-6789-7000-8000-000000000099/preview?format=markdown&max_lines=500&file_path=src%2Fmain.rs",
      expect.objectContaining({ method: "GET" })
    );
    expect(result.job_id).toBe("018f2345-6789-7000-8000-000000000099");
    expect(result.format).toBe("markdown");
    expect(result.content).toContain("Live Preview");
    expect(result.files).toHaveLength(1);
    expect(result.files[0].path).toBe("src/main.rs");
  });

  it("throws ApiClientError when getJobPreview returns 404 Problem Details", async () => {
    const notFoundProblem: ApiProblemDetails = {
      type: "https://errors.web-repo-to-ai.org/not-found",
      title: "Resource Not Found",
      status: 404,
      detail: "Job not found",
      request_id: "req-preview-404",
    };

    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      headers: new Headers({ "content-type": "application/problem+json" }),
      json: async () => notFoundProblem,
    });

    const client = new ApiClient({ fetchFn: mockFetch });
    await expect(client.getJobPreview("non-existent-id")).rejects.toThrow("Job not found");
  });
});

describe("SvelteKit Route API Wiring Verification", () => {
  it("+page.svelte imports and invokes defaultApiClient without mock setTimeout", () => {
    const pagePath = resolve(process.cwd(), "src/routes/+page.svelte");
    const pageContent = readFileSync(pagePath, "utf-8");

    expect(pageContent).toContain("defaultApiClient");
    expect(pageContent).toContain("defaultApiClient.createJob");
    expect(pageContent).toContain("/workspace?jobId=");
    expect(pageContent).not.toContain("setTimeout(() => {");
  });

  it("workspace/+page.svelte reads jobId and queries defaultApiClient.getJob", () => {
    const workspacePagePath = resolve(process.cwd(), "src/routes/workspace/+page.svelte");
    const workspaceContent = readFileSync(workspacePagePath, "utf-8");

    expect(workspaceContent).toContain("defaultApiClient");
    expect(workspaceContent).toContain("defaultApiClient.getJob");
    expect(workspaceContent).toContain("mapApiJobToConversionJob");
    expect(workspaceContent).toContain("params.get('jobId')");
  });
});
