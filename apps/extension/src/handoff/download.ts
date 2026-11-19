// File download dispatcher for web-repo-to-ai Browser Extension

import { OutputFormat } from "../types/index";

/**
 * Maps an OutputFormat to standard MIME content types.
 */
export function getMimeTypeForFormat(format: OutputFormat): string {
  switch (format) {
    case "markdown":
      return "text/markdown;charset=utf-8";
    case "txt":
      return "text/plain;charset=utf-8";
    case "json":
      return "application/json;charset=utf-8";
    case "html":
      return "text/html;charset=utf-8";
    case "jsonl":
      return "application/x-ndjson;charset=utf-8";
    case "zip":
      return "application/zip";
    default:
      return "text/plain;charset=utf-8";
  }
}

/**
 * Maps an OutputFormat to standard file extensions.
 */
export function getExtensionForFormat(format: OutputFormat): string {
  switch (format) {
    case "markdown":
      return "md";
    case "txt":
      return "txt";
    case "json":
      return "json";
    case "html":
      return "html";
    case "jsonl":
      return "jsonl";
    case "zip":
      return "zip";
    default:
      return "txt";
  }
}

/**
 * Sanitizes a title or URL into a safe filesystem filename.
 */
export function sanitizeDownloadFilename(
  titleOrUrl: string,
  jobId: string,
  format: OutputFormat
): string {
  const shortId = jobId ? jobId.replace(/-/g, "").slice(0, 8) : "doc";
  const ext = getExtensionForFormat(format);

  if (!titleOrUrl || typeof titleOrUrl !== "string") {
    return `converted-${shortId}.${ext}`;
  }

  // If input looks like URL, extract hostname + path
  let baseName = titleOrUrl;
  try {
    const parsed = new URL(titleOrUrl);
    baseName = `${parsed.hostname}-${parsed.pathname}`;
  } catch {
    // Treat as title string
  }

  const sanitized = baseName
    .toLowerCase()
    .replace(/https?:\/\//g, "")
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 40);

  return `${sanitized || "content"}-${shortId}.${ext}`;
}

/**
 * Triggers a native file download using chrome.downloads or synthetic <a> click.
 */
export async function triggerFileDownload(
  content: string,
  filename: string,
  format: OutputFormat
): Promise<void> {
  const mimeType = getMimeTypeForFormat(format);

  if (typeof Blob === "undefined" || typeof URL === "undefined") {
    return;
  }

  const blob = new Blob([content], { type: mimeType });
  const objectUrl = URL.createObjectURL(blob);

  // Chrome Extension Downloads API
  if (
    typeof chrome !== "undefined" &&
    chrome.downloads &&
    typeof chrome.downloads.download === "function"
  ) {
    try {
      await chrome.downloads.download({
        url: objectUrl,
        filename,
        conflictAction: "uniquify",
        saveAs: false,
      });
      // Allow slight buffer before revoking blob URL
      setTimeout(() => URL.revokeObjectURL(objectUrl), 10000);
      return;
    } catch (err) {
      console.warn("chrome.downloads.download failed, falling back to anchor click:", err);
    }
  }

  // DOM Anchor click fallback
  if (typeof document !== "undefined" && document.createElement) {
    const a = document.createElement("a");
    a.href = objectUrl;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    setTimeout(() => URL.revokeObjectURL(objectUrl), 2000);
  }
}
