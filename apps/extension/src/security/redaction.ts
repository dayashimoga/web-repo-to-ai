// Secret Redaction and Sensitive Data Masking for web-repo-to-ai browser extension

/**
 * Masks an API key for safe presentation in user interfaces or logs,
 * adhering to the project's zero-secret-leakage security contract.
 *
 * Examples:
 *   - "wr_live_abcdef1234567890abcdef" -> "wr_live_••••cdef"
 *   - "secret-token-12345" -> "secr••••45"
 *   - "short" -> "••••••••"
 */
export function maskApiKey(apiKey?: string | null): string {
  if (!apiKey || typeof apiKey !== "string") {
    return "";
  }

  const trimmed = apiKey.trim();
  if (!trimmed) {
    return "";
  }

  if (trimmed.startsWith("wr_live_")) {
    const suffix = trimmed.slice(-4);
    return `wr_live_••••${suffix}`;
  }

  if (trimmed.length > 8) {
    const prefix = trimmed.slice(0, 4);
    const suffix = trimmed.slice(-2);
    return `${prefix}••••${suffix}`;
  }

  return "••••••••";
}

const SENSITIVE_PATTERNS: ReadonlyArray<{ pattern: RegExp; replacement: string }> = [
  // web-repo-to-ai API key pattern
  { pattern: /wr_live_[a-zA-Z0-9]{8,}/g, replacement: "wr_live_••••[REDACTED]" },
  // Bearer token authentication header pattern
  { pattern: /Bearer\s+[a-zA-Z0-9_\-\.]{8,}/gi, replacement: "Bearer [REDACTED]" },
  // Embedded HTTP user:password credentials in URLs
  { pattern: /(https?:\/\/)([^:@\s]+):([^@\s]+)@/gi, replacement: "$1[REDACTED]:[REDACTED]@" },
  // Generic API key / token query params in URLs
  {
    pattern: /([?&](?:api_key|apiKey|token|secret|password)=)[^&\s]+/gi,
    replacement: "$1[REDACTED]",
  },
];

/**
 * Sanitizes an error object or error string, stripping out any embedded
 * API keys, authorization bearer tokens, or sensitive credentials.
 */
export function sanitizeError(error: unknown): string {
  let message: string;
  if (!error) {
    return "Unknown error";
  }

  if (typeof error === "string") {
    message = error;
  } else if (error instanceof Error) {
    message = error.message;
  } else if (typeof error === "object" && "message" in (error as any)) {
    message = String((error as any).message);
  } else {
    try {
      message = JSON.stringify(error);
    } catch {
      message = String(error);
    }
  }

  let sanitized = message;
  for (const { pattern, replacement } of SENSITIVE_PATTERNS) {
    sanitized = sanitized.replace(pattern, replacement);
  }

  return sanitized;
}

/**
 * Sanitizes a log message before sending to console or debugging output.
 */
export function sanitizeLog(message: string): string {
  if (!message || typeof message !== "string") {
    return "";
  }

  let sanitized = message;
  for (const { pattern, replacement } of SENSITIVE_PATTERNS) {
    sanitized = sanitized.replace(pattern, replacement);
  }

  return sanitized;
}

const SENSITIVE_HEADER_NAMES: ReadonlySet<string> = new Set([
  "authorization",
  "x-api-key",
  "cookie",
  "set-cookie",
  "proxy-authorization",
]);

/**
 * Checks if a given HTTP header name carries sensitive credential data.
 */
export function isSensitiveHeader(headerName: string): boolean {
  if (!headerName || typeof headerName !== "string") {
    return false;
  }
  return SENSITIVE_HEADER_NAMES.has(headerName.trim().toLowerCase());
}
