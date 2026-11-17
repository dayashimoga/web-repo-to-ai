/**
 * Strongly-Typed REST API Client for `services/api` (`apps/web/src/lib/api/client.ts`)
 *
 * Implements:
 * - Job creation (`POST /api/v1/jobs`)
 * - Job inspection (`GET /api/v1/jobs/{job_id}`)
 * - Source URL detection (`POST /api/v1/sources/detect`)
 * - Direct artifact download URLs (`GET /api/v1/jobs/{job_id}/artifacts/{format}`)
 * - RFC 7807 Problem Details decoding and strongly-typed error handling.
 */

import type {
  ApiJobResponse,
  ApiProblemDetails,
  CreateJobPayload,
  DetectSourcesPayload,
  DetectSourcesResponse,
  JobPreviewResponse,
} from "./types";

/** Strongly-typed error representing an RFC 7807 API failure or transport error. */
export class ApiClientError extends Error {
  readonly status: number;
  readonly problem?: ApiProblemDetails;
  readonly requestId?: string;

  constructor(message: string, status: number, problem?: ApiProblemDetails) {
    super(message);
    this.name = "ApiClientError";
    this.status = status;
    this.problem = problem;
    this.requestId = problem?.request_id;
  }
}

export interface ApiClientConfig {
  baseUrl?: string;
  fetchFn?: typeof fetch;
  defaultHeaders?: Record<string, string>;
}

export class ApiClient {
  private readonly baseUrl: string;
  private readonly fetchFn: typeof fetch;
  private readonly defaultHeaders: Record<string, string>;

  constructor(config?: ApiClientConfig) {
    // Determine default base URL: browser relative path `/api/v1` or configured origin
    const configuredBase =
      config?.baseUrl ||
      (typeof process !== "undefined" && process.env?.VITE_API_BASE_URL) ||
      "/api/v1";

    this.baseUrl = configuredBase.replace(/\/+$/, "");
    this.fetchFn =
      config?.fetchFn || (typeof window !== "undefined" ? window.fetch.bind(window) : fetch);
    this.defaultHeaders = {
      Accept: "application/json, application/problem+json",
      "Content-Type": "application/json",
      ...config?.defaultHeaders,
    };
  }

  /**
   * Dispatches a new conversion job request (`POST /api/v1/jobs`).
   *
   * @throws {ApiClientError} on RFC 7807 error or network failure.
   */
  async createJob(payload: CreateJobPayload): Promise<ApiJobResponse> {
    return this.request<ApiJobResponse>("/jobs", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  }

  /**
   * Retrieves status and metadata for a conversion job by UUID (`GET /api/v1/jobs/{job_id}`).
   *
   * @throws {ApiClientError} on RFC 7807 error or network failure.
   */
  async getJob(jobId: string): Promise<ApiJobResponse> {
    const sanitizedId = encodeURIComponent(jobId.trim());
    return this.request<ApiJobResponse>(`/jobs/${sanitizedId}`, {
      method: "GET",
    });
  }

  /**
   * Classifies candidate source URLs via SSRF validator and detector (`POST /api/v1/sources/detect`).
   *
   * @throws {ApiClientError} on RFC 7807 error or network failure.
   */
  async detectSources(urls: string[]): Promise<DetectSourcesResponse> {
    const payload: DetectSourcesPayload = { urls };
    return this.request<DetectSourcesResponse>("/sources/detect", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  }

  /**
   * Returns the direct artifact download URL for a given job and format.
   */
  getArtifactUrl(jobId: string, format: string): string {
    const sanitizedId = encodeURIComponent(jobId.trim());
    const sanitizedFmt = encodeURIComponent(format.trim().toLowerCase());
    return `${this.baseUrl}/jobs/${sanitizedId}/artifacts/${sanitizedFmt}`;
  }

  /**
   * Returns the Server-Sent Events (SSE) progress streaming URL for a given job.
   */
  getStreamUrl(jobId: string): string {
    const sanitizedId = encodeURIComponent(jobId.trim());
    return `${this.baseUrl}/jobs/${sanitizedId}/stream`;
  }

  /**
   * Retrieves a paginated / windowed preview for a conversion job (`GET /api/v1/jobs/{job_id}/preview`).
   *
   * @throws {ApiClientError} on RFC 7807 error or network failure.
   */
  async getJobPreview(
    jobId: string,
    options?: { format?: string; maxLines?: number; filePath?: string }
  ): Promise<JobPreviewResponse> {
    const sanitizedId = encodeURIComponent(jobId.trim());
    const params = new URLSearchParams();
    if (options?.format) params.set("format", options.format);
    if (options?.maxLines) params.set("max_lines", String(options.maxLines));
    if (options?.filePath) params.set("file_path", options.filePath);

    const queryStr = params.toString();
    const endpoint = `/jobs/${sanitizedId}/preview${queryStr ? `?${queryStr}` : ""}`;
    return this.request<JobPreviewResponse>(endpoint, { method: "GET" });
  }

  /**
   * Internal request dispatcher with RFC 7807 Problem Details decoding.
   */
  private async request<T>(path: string, init: RequestInit): Promise<T> {
    const targetUrl = `${this.baseUrl}${path.startsWith("/") ? path : `/${path}`}`;
    const headers = {
      ...this.defaultHeaders,
      ...init.headers,
    };

    let response: Response;
    try {
      response = await this.fetchFn(targetUrl, {
        ...init,
        headers,
      });
    } catch (networkError) {
      const msg = networkError instanceof Error ? networkError.message : String(networkError);
      throw new ApiClientError(
        `Failed to connect to conversion API service (${targetUrl}): ${msg}`,
        0
      );
    }

    if (!response.ok) {
      let problem: ApiProblemDetails | undefined;
      let errorMsg = `API request failed with HTTP ${response.status} ${response.statusText}`;

      const contentType = response.headers.get("content-type") || "";
      if (contentType.includes("json") || contentType.includes("problem")) {
        try {
          const body = await response.json();
          if (body && typeof body === "object") {
            problem = body as ApiProblemDetails;
            if (problem.detail) {
              errorMsg = problem.detail;
            } else if (problem.title) {
              errorMsg = problem.title;
            }
          }
        } catch {
          // Fall back to default error message if parsing fails
        }
      }

      throw new ApiClientError(errorMsg, response.status, problem);
    }

    // Parse successful JSON payload
    try {
      const data = await response.json();
      return data as T;
    } catch (parseError) {
      const msg = parseError instanceof Error ? parseError.message : String(parseError);
      throw new ApiClientError(`Failed to parse API response JSON: ${msg}`, response.status);
    }
  }
}

/** Global default API client instance. */
export const defaultApiClient = new ApiClient();
