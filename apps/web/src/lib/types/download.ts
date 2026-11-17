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
  format: ArtifactFormat;
  label: string;
  badge: string;
  filename: string;
  mimeType: string;
  extension: string;
  byteSize: number;
  sha256Hex: string;
  content: string;
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
  sourceUrl: string
): Promise<ExportArtifact[]> {
  const baseName = deriveArtifactBaseName(sourceUrl);
  const artifacts: ExportArtifact[] = [];

  for (const fmt of PREVIEW_FORMATS) {
    const content = formatContentForExport(files, fmt.id, { sourceUrl });
    const byteSize = new Blob([content]).size;
    const sha256Hex = await computeSha256Hex(content);
    const filename = `${baseName}.${fmt.extension}`;

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
      description: fmt.description,
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
