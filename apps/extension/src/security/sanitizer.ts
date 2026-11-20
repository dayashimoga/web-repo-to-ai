// HTML & DOM Sanitization for web-repo-to-ai browser extension

const DANGEROUS_TAGS: ReadonlyArray<string> = [
  "script",
  "style",
  "noscript",
  "iframe",
  "object",
  "embed",
  "applet",
  "svg",
  "canvas",
  "audio",
  "video",
  "template",
  "base",
  "form",
  "input",
  "textarea",
  "button",
  "select",
  "option",
];

const DANGEROUS_PROTOCOLS: ReadonlyArray<string> = [
  "javascript:",
  "vbscript:",
  "data:text/html",
  "data:text/javascript",
  "data:application/javascript",
];

/**
 * Escapes common HTML special characters to prevent HTML injection / XSS.
 */
export function escapeHtml(str: string): string {
  if (!str || typeof str !== "string") {
    return "";
  }
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/**
 * Strips dangerous inline event handlers (on*) and malicious link protocols
 * (javascript:, vbscript:, data:text/html) from a DOM Element in-place.
 */
export function stripDangerousAttributes(el: Element): void {
  const attrs = Array.from(el.attributes || []);
  for (const attr of attrs) {
    const name = attr.name.toLowerCase();
    const value = (attr.value || "").trim().toLowerCase();

    // Remove inline event handlers
    if (name.startsWith("on")) {
      el.removeAttribute(attr.name);
      continue;
    }

    // Remove dangerous attribute schemes
    if (name === "href" || name === "src" || name === "action" || name === "formaction") {
      for (const dangerous of DANGEROUS_PROTOCOLS) {
        if (value.startsWith(dangerous)) {
          el.removeAttribute(attr.name);
          break;
        }
      }
    }

    // Remove srcdoc attribute
    if (name === "srcdoc") {
      el.removeAttribute(attr.name);
    }
  }
}

/**
 * Sanitizes an HTML string by parsing it into a detached DOM document,
 * stripping all executable script tags, active media, forms, and event handlers,
 * and returning clean HTML. Falls back to regex-based tag stripping if no DOM is available.
 */
export function sanitizeHtmlContent(dirtyHtml: string): string {
  if (!dirtyHtml || typeof dirtyHtml !== "string") {
    return "";
  }

  // DOMParser path when available (Browser & Vitest DOM environments)
  if (typeof DOMParser !== "undefined") {
    try {
      const parser = new DOMParser();
      const doc = parser.parseFromString(dirtyHtml, "text/html");

      // Strip dangerous tags
      for (const tag of DANGEROUS_TAGS) {
        doc.querySelectorAll(tag).forEach((el) => el.parentNode?.removeChild(el));
      }

      // Strip aria-hidden elements
      doc.querySelectorAll("[aria-hidden='true']").forEach((el) => el.parentNode?.removeChild(el));

      // Strip dangerous attributes from all elements
      doc.querySelectorAll("*").forEach((el) => stripDangerousAttributes(el));

      return doc.body.innerHTML.trim();
    } catch {
      // Fallback to regex path
    }
  }

  // Regex fallback for non-DOM environments
  let sanitized = dirtyHtml;

  // Remove dangerous tags and their contents
  for (const tag of DANGEROUS_TAGS) {
    const tagRegex = new RegExp(`<${tag}\\b[^>]*>[\\s\\S]*?<\\/${tag}>`, "gi");
    sanitized = sanitized.replace(tagRegex, "");
    // Also remove self-closing or unclosed variants
    const selfClosing = new RegExp(`<${tag}\\b[^>]*\\/?>`, "gi");
    sanitized = sanitized.replace(selfClosing, "");
  }

  // Remove inline on* event handlers
  sanitized = sanitized.replace(/\s+on[a-zA-Z]+\s*=\s*(["'][^"']*["']|[^\s>]+)/gi, "");

  // Remove javascript: and data:text/html links
  sanitized = sanitized.replace(
    /\s+(?:href|src|action|formaction)\s*=\s*["']\s*(?:javascript|vbscript|data:text\/html)[^"']*["']/gi,
    ""
  );

  // Remove srcdoc
  sanitized = sanitized.replace(/\s+srcdoc\s*=\s*(["'][^"']*["']|[^\s>]+)/gi, "");

  return sanitized.trim();
}
