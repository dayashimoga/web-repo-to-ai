// Security Policy Enforcement for web-repo-to-ai browser extension

/**
 * Canonical Manifest V3 Content Security Policy for extension pages.
 */
export const EXTENSION_CSP = "script-src 'self'; object-src 'self'; worker-src 'self'";

/**
 * List of internal or privileged browser URL schemes that cannot and must not
 * be targeted for script injection or external API conversion.
 */
export const PRIVILEGED_SCHEMES: ReadonlyArray<string> = [
  "chrome:",
  "chrome-extension:",
  "edge:",
  "about:",
  "devtools:",
  "view-source:",
  "javascript:",
  "data:",
  "file:",
];

/**
 * Checks whether a given URL is an internal or privileged browser page.
 */
export function isPrivilegedUrl(urlString?: string | null): boolean {
  if (!urlString || typeof urlString !== "string") {
    return true; // Treat null/empty as privileged/blocked
  }

  const trimmed = urlString.trim().toLowerCase();

  for (const scheme of PRIVILEGED_SCHEMES) {
    if (trimmed.startsWith(scheme)) {
      return true;
    }
  }

  try {
    const parsed = new URL(trimmed);
    return PRIVILEGED_SCHEMES.includes(parsed.protocol);
  } catch {
    // If it fails URL parsing and is not an HTTP/HTTPS URL, treat as privileged/blocked
    return true;
  }
}

/**
 * Checks whether a given URL uses a safe HTTP or HTTPS protocol.
 */
export function isSafeHttpUrl(urlString?: string | null): boolean {
  if (!urlString || typeof urlString !== "string") {
    return false;
  }

  const trimmed = urlString.trim();
  // Reject URLs containing null bytes, CRLF, or other dangerous control characters
  if (/[\x00-\x1F\x7F]/.test(trimmed)) {
    return false;
  }

  try {
    const parsed = new URL(trimmed);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}

export interface UrlValidationResult {
  valid: boolean;
  reason?: string;
  sanitizedUrl?: string;
}

/**
 * Validates a user-configured API Base URL to ensure it is well-formed, safe,
 * and does not contain dangerous protocols, user credentials, or fragments.
 */
export function validateApiBaseUrl(apiUrl?: string | null): UrlValidationResult {
  if (!apiUrl || typeof apiUrl !== "string" || !apiUrl.trim()) {
    return { valid: false, reason: "API URL cannot be empty." };
  }

  const trimmed = apiUrl.trim();

  // Control character check
  if (/[\x00-\x1F\x7F]/.test(trimmed)) {
    return { valid: false, reason: "API URL contains illegal control characters." };
  }

  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    return { valid: false, reason: "API URL must be a valid, well-formed URL." };
  }

  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    return { valid: false, reason: "API URL must use http or https protocol." };
  }

  if (parsed.username || parsed.password) {
    return { valid: false, reason: "API URL must not contain embedded user credentials." };
  }

  if (parsed.hash) {
    return { valid: false, reason: "API URL must not include URL fragments (#)." };
  }

  // Remove trailing slashes for clean consistency
  const sanitized = trimmed.replace(/\/+$/, "");

  return { valid: true, sanitizedUrl: sanitized };
}

/**
 * Validates a target web URL before dispatching for DOM extraction or backend conversion.
 */
export function validateTargetUrl(targetUrl?: string | null): UrlValidationResult {
  if (!targetUrl || typeof targetUrl !== "string" || !targetUrl.trim()) {
    return { valid: false, reason: "Target URL cannot be empty." };
  }

  const trimmed = targetUrl.trim();

  // Check for privileged browser schemes
  if (isPrivilegedUrl(trimmed)) {
    return {
      valid: false,
      reason: "Cannot convert privileged or internal browser pages (e.g. chrome://, about:).",
    };
  }

  if (!isSafeHttpUrl(trimmed)) {
    return {
      valid: false,
      reason: "Target URL must be a valid http:// or https:// web resource.",
    };
  }

  return { valid: true, sanitizedUrl: trimmed };
}
