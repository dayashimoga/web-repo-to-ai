// Workspace deep-linking and browser navigation for web-repo-to-ai Browser Extension

import { OutputFormat } from "../types/index";

/**
 * Constructs the deep-link URL to view a completed conversion job in the Web UI workspace.
 */
export function buildWorkspaceUrl(baseUrl: string, jobId: string): string {
  const normalized = baseUrl.replace(/\/+$/, "");
  return `${normalized}/workspace?job_id=${encodeURIComponent(jobId)}`;
}

/**
 * Constructs the deep-link URL to inspect a raw job preview.
 */
export function buildPreviewUrl(baseUrl: string, jobId: string, format?: OutputFormat): string {
  const normalized = baseUrl.replace(/\/+$/, "");
  const query = format ? `?format=${encodeURIComponent(format)}` : "";
  return `${normalized}/jobs/${encodeURIComponent(jobId)}/preview${query}`;
}

/**
 * Constructs the landing URL pre-populated with a source URL and format.
 */
export function buildPreFilledUrl(
  baseUrl: string,
  targetUrl: string,
  format?: OutputFormat
): string {
  const normalized = baseUrl.replace(/\/+$/, "");
  const params = new URLSearchParams();
  params.set("source", targetUrl);
  if (format) {
    params.set("format", format);
  }
  return `${normalized}/?${params.toString()}`;
}

/**
 * Navigates the browser to the target URL, using chrome.tabs or window.open.
 */
export async function navigateToUrl(url: string): Promise<void> {
  if (typeof chrome !== "undefined" && chrome.tabs && typeof chrome.tabs.create === "function") {
    try {
      await chrome.tabs.create({ url, active: true });
      return;
    } catch (err) {
      console.warn("chrome.tabs.create failed, falling back to window.open:", err);
    }
  }

  if (typeof window !== "undefined" && window.open) {
    window.open(url, "_blank");
  }
}
