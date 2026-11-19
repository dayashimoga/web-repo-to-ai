// DOM and Metadata Extraction Engine for web-repo-to-ai Browser Extension

import { PageHeading, PageSnapshot } from "../types/index";

/**
 * Strips script tags, styles, noscript, and invisible elements from an HTML tree,
 * returning sanitized HTML and text content.
 */
export function sanitizeDocumentContent(doc: Document): {
  html: string;
  textContent: string;
} {
  // Clone body or root document element to avoid modifying the active DOM
  const root = doc.body
    ? (doc.body.cloneNode(true) as HTMLElement)
    : (doc.documentElement.cloneNode(true) as HTMLElement);

  // Blacklisted elements that contain code, styling, or non-textual embeds
  const removeSelectors = [
    "script",
    "style",
    "noscript",
    "iframe",
    "object",
    "embed",
    "svg",
    "canvas",
    "audio",
    "video",
    "template",
    "[aria-hidden='true']",
  ];

  for (const selector of removeSelectors) {
    const elements = root.querySelectorAll(selector);
    elements.forEach((el) => el.parentNode?.removeChild(el));
  }

  // Remove inline event handlers
  const allElements = root.querySelectorAll("*");
  allElements.forEach((el) => {
    const attrs = Array.from(el.attributes || []);
    for (const attr of attrs) {
      if (attr.name.toLowerCase().startsWith("on")) {
        el.removeAttribute(attr.name);
      }
    }
  });

  const html = root.innerHTML.trim();
  const textContent = (root.textContent || "").replace(/\s+/g, " ").trim();

  return { html, textContent };
}

/**
 * In-page extraction function that runs inside document context to produce a PageSnapshot.
 */
export function extractDOMFromDocument(doc: Document, fallbackUrl?: string): PageSnapshot {
  const url = fallbackUrl || (doc.location ? doc.location.href : "");
  const title = doc.title ? doc.title.trim() : "";

  // Canonical link
  const canonicalEl = doc.querySelector('link[rel="canonical"]');
  const canonicalUrl = canonicalEl ? canonicalEl.getAttribute("href") || undefined : undefined;

  // Description
  const descEl =
    doc.querySelector('meta[name="description"]') ||
    doc.querySelector('meta[property="og:description"]');
  const description = descEl ? descEl.getAttribute("content") || undefined : undefined;

  // Author
  const authorEl =
    doc.querySelector('meta[name="author"]') || doc.querySelector('meta[name="twitter:creator"]');
  const author = authorEl ? authorEl.getAttribute("content") || undefined : undefined;

  // Headings
  const headings: PageHeading[] = [];
  const headingElements = doc.querySelectorAll("h1, h2, h3, h4, h5, h6");
  headingElements.forEach((el) => {
    const text = (el.textContent || "").trim();
    if (text) {
      const level = parseInt(el.tagName.substring(1), 10) || 1;
      headings.push({ level, text });
    }
  });

  const { html, textContent } = sanitizeDocumentContent(doc);
  const charCount = textContent.length;
  // Standard 4 chars per token heuristic
  const estimatedTokens = Math.ceil(charCount / 4);

  return {
    url,
    title,
    html,
    textContent,
    charCount,
    estimatedTokens,
    canonicalUrl,
    description,
    author,
    headings,
    capturedAt: new Date().toISOString(),
  };
}

/**
 * Captures the sanitized DOM snapshot and metadata from the given tab ID
 * using chrome.scripting.executeScript.
 */
export async function captureActiveTabDOM(tabId?: number): Promise<PageSnapshot> {
  if (
    typeof chrome !== "undefined" &&
    chrome.scripting &&
    typeof chrome.scripting.executeScript === "function" &&
    tabId !== undefined
  ) {
    try {
      const results = await chrome.scripting.executeScript({
        target: { tabId },
        func: () => {
          // Self-contained in-page execution
          const doc = document;
          const url = window.location.href;
          const title = doc.title || "";

          const canonicalEl = doc.querySelector('link[rel="canonical"]');
          const canonicalUrl = canonicalEl
            ? canonicalEl.getAttribute("href") || undefined
            : undefined;

          const descEl =
            doc.querySelector('meta[name="description"]') ||
            doc.querySelector('meta[property="og:description"]');
          const description = descEl ? descEl.getAttribute("content") || undefined : undefined;

          const authorEl =
            doc.querySelector('meta[name="author"]') ||
            doc.querySelector('meta[name="twitter:creator"]');
          const author = authorEl ? authorEl.getAttribute("content") || undefined : undefined;

          const headings: Array<{ level: number; text: string }> = [];
          doc.querySelectorAll("h1, h2, h3, h4, h5, h6").forEach((el) => {
            const text = (el.textContent || "").trim();
            if (text) {
              headings.push({
                level: parseInt(el.tagName.substring(1), 10) || 1,
                text,
              });
            }
          });

          // Clean DOM
          const root = doc.body
            ? (doc.body.cloneNode(true) as HTMLElement)
            : (doc.documentElement.cloneNode(true) as HTMLElement);
          const removeSelectors = [
            "script",
            "style",
            "noscript",
            "iframe",
            "object",
            "embed",
            "svg",
            "canvas",
            "audio",
            "video",
            "template",
            "[aria-hidden='true']",
          ];
          for (const s of removeSelectors) {
            root.querySelectorAll(s).forEach((el) => el.parentNode?.removeChild(el));
          }

          root.querySelectorAll("*").forEach((el) => {
            const attrs = Array.from(el.attributes || []);
            for (const a of attrs) {
              if (a.name.toLowerCase().startsWith("on")) {
                el.removeAttribute(a.name);
              }
            }
          });

          const html = root.innerHTML.trim();
          const textContent = (root.textContent || "").replace(/\s+/g, " ").trim();
          const charCount = textContent.length;
          const estimatedTokens = Math.ceil(charCount / 4);

          return {
            url,
            title,
            html,
            textContent,
            charCount,
            estimatedTokens,
            canonicalUrl,
            description,
            author,
            headings,
            capturedAt: new Date().toISOString(),
          };
        },
      });

      if (results && results.length > 0 && results[0].result) {
        return results[0].result as PageSnapshot;
      }
    } catch (err: any) {
      console.warn("chrome.scripting.executeScript failed, falling back to local DOM", err);
    }
  }

  // Fallback for document in context (e.g. testing)
  if (typeof document !== "undefined") {
    return extractDOMFromDocument(document);
  }

  // Minimal empty snapshot fallback
  return {
    url: "",
    title: "Untitled Page",
    html: "",
    textContent: "",
    charCount: 0,
    estimatedTokens: 0,
    headings: [],
    capturedAt: new Date().toISOString(),
  };
}
