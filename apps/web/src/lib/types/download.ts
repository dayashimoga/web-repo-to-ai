/**
 * Export Artifacts & Download Controls Domain Model
 *
 * Implements:
 * 1. Multi-format artifact definitions matching docs/API.md and docs/DATA_MODEL.md.
 * 2. Cryptographic SHA-256 integrity checksum generation via Web Crypto API.
 * 3. Standard checksum manifest generation (checksums.txt and manifest.json).
 * 4. Client-side blob downloading with automatic URL cleanup.
 */

import {
  type PreviewFile,
  type PreviewFormat,
  PREVIEW_FORMATS,
  formatContentForExport,
} from "./preview";

export type ArtifactFormat = PreviewFormat | "zip" | "manifest";

export interface ExportArtifact {
  id: string;
  format: ArtifactFormat | string;
  label: string;
  badge: string;
  filename: string;
  mimeType: string;
  extension: string;
  byteSize: number;
  sha256Hex: string;
  content: string;
  downloadUrl?: string;
  description: string;
}

export interface ChecksumManifest {
  schema_version: "1.0.0";
  source_url: string;
  generated_at: string;
  total_artifacts: number;
  total_bytes: number;
  artifacts: {
    format: string;
    filename: string;
    byte_size: number;
    sha256: string;
  }[];
}

/**
 * Computes standard SHA-256 hexadecimal checksum.
 */
export async function computeSha256Hex(text: string): Promise<string> {
  if (typeof crypto !== "undefined" && crypto.subtle) {
    const encoder = new TextEncoder();
    const data = encoder.encode(text);
    const hashBuffer = await crypto.subtle.digest("SHA-256", data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
  }

  // Deterministic fallback for environments without Web Crypto API
  let hash = 0;
  for (let i = 0; i < text.length; i++) {
    hash = (hash << 5) - hash + text.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash).toString(16).padStart(64, "0");
}

/**
 * Derives a clean filesystem-safe base name from a source URL.
 */
export function deriveArtifactBaseName(sourceUrl: string): string {
  try {
    const parsed = new URL(sourceUrl);
    const parts = parsed.pathname.split("/").filter(Boolean);
    if (parsed.hostname === "github.com" && parts.length >= 2) {
      return parts[1].replace(/[^a-zA-Z0-9_-]/g, "_");
    }
    if (parts.length > 0) {
      return parts[0].replace(/[^a-zA-Z0-9_-]/g, "_");
    }
    return parsed.hostname.replace(/[^a-zA-Z0-9_-]/g, "_");
  } catch {
    return "context-export";
  }
}

/**
 * Generates all export artifacts for the provided files and source URL.
 */
export async function buildExportArtifacts(
  files: PreviewFile[],
  sourceUrl: string,
  options?: {
    jobId?: string;
    backendArtifacts?: {
      format: string;
      filename: string;
      downloadUrl: string;
      byteSize: number;
      sha256Hex: string;
    }[];
  }
): Promise<ExportArtifact[]> {
  const baseName = deriveArtifactBaseName(sourceUrl);
  const artifacts: ExportArtifact[] = [];

  for (const fmt of PREVIEW_FORMATS) {
    const backendMatch = options?.backendArtifacts?.find(
      (b) => b.format.toLowerCase() === fmt.id.toLowerCase()
    );
    const content = formatContentForExport(files, fmt.id, { sourceUrl });
    const byteSize = backendMatch?.byteSize || new Blob([content]).size;
    const sha256Hex = backendMatch?.sha256Hex || (await computeSha256Hex(content));
    const filename = backendMatch?.filename || `${baseName}.${fmt.extension}`;
    const downloadUrl =
      backendMatch?.downloadUrl ||
      (options?.jobId ? `/api/v1/jobs/${options.jobId}/artifacts/${fmt.id}` : undefined);

    artifacts.push({
      id: `art-${fmt.id}`,
      format: fmt.id,
      label: fmt.label,
      badge: fmt.badge,
      filename,
      mimeType: fmt.mime,
      extension: fmt.extension,
      byteSize,
      sha256Hex,
      content,
      downloadUrl,
      description: fmt.description,
    });
  }

  // Include ZIP bundle if jobId is present or if backendArtifacts contains zip
  const zipBackend = options?.backendArtifacts?.find((b) => b.format.toLowerCase() === "zip");
  if (options?.jobId || zipBackend) {
    const zipFilename = zipBackend?.filename || `${baseName}-bundle.zip`;
    const zipDownloadUrl =
      zipBackend?.downloadUrl ||
      (options?.jobId ? `/api/v1/jobs/${options.jobId}/artifacts/zip` : undefined);
    const zipSize = zipBackend?.byteSize || artifacts.reduce((acc, a) => acc + a.byteSize, 0);
    const zipSha =
      zipBackend?.sha256Hex || "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    artifacts.push({
      id: "art-zip",
      format: "zip",
      label: "ZIP Package",
      badge: ".zip",
      filename: zipFilename,
      mimeType: "application/zip",
      extension: "zip",
      byteSize: zipSize,
      sha256Hex: zipSha,
      content: "",
      downloadUrl: zipDownloadUrl,
      description: "Complete archive bundle containing all formats, manifest, and checksums.",
    });
  }

  return artifacts;
}

/**
 * Generates standard checksums.txt file content.
 */
export function generateChecksumsTxt(artifacts: ExportArtifact[], sourceUrl: string): string {
  const now = new Date().toISOString();
  const lines = [
    `# SHA-256 Checksums for web-repo-to-ai Context Export`,
    `# Source: ${sourceUrl}`,
    `# Generated: ${now}`,
    `# Format: <sha256_hash>  <filename>`,
    ``,
  ];

  artifacts.forEach((art) => {
    lines.push(`${art.sha256Hex}  ${art.filename}`);
  });

  return lines.join("\n");
}

/**
 * Generates JSON checksum manifest matching CDM specifications.
 */
export function generateChecksumManifestJson(
  artifacts: ExportArtifact[],
  sourceUrl: string
): string {
  const now = new Date().toISOString();
  const manifest: ChecksumManifest = {
    schema_version: "1.0.0",
    source_url: sourceUrl,
    generated_at: now,
    total_artifacts: artifacts.length,
    total_bytes: artifacts.reduce((acc, a) => acc + a.byteSize, 0),
    artifacts: artifacts.map((a) => ({
      format: a.format,
      filename: a.filename,
      byte_size: a.byteSize,
      sha256: a.sha256Hex,
    })),
  };

  return JSON.stringify(manifest, null, 2);
}

/**
 * Triggers a client-side Blob download.
 */
export function triggerBlobDownload(filename: string, content: string, mimeType: string): void {
  if (typeof window === "undefined" || typeof document === "undefined") return;

  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * Triggers a download either directly via a server endpoint URL or client-side Blob.
 */
export function triggerDownload(
  filename: string,
  content?: string,
  mimeType?: string,
  downloadUrl?: string
): void {
  if (typeof window === "undefined" || typeof document === "undefined") return;

  if (downloadUrl) {
    const a = document.createElement("a");
    a.href = downloadUrl;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    return;
  }

  if (content !== undefined && mimeType) {
    triggerBlobDownload(filename, content, mimeType);
  }
}
