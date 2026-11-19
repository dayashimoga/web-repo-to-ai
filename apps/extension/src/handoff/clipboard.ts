// Clipboard copy utility and prompt formatter for web-repo-to-ai Browser Extension

import { CopyOptions, OutputFormat } from "../types/index";

/**
 * Formats converted content for clipboard copying based on the selected mode.
 */
export function formatContentForClipboard(
  content: string,
  url: string,
  title: string,
  format: OutputFormat,
  options?: CopyOptions
): string {
  const mode = options?.mode || "raw";

  switch (mode) {
    case "prompt": {
      const codeLang = format === "markdown" ? "markdown" : format;
      const prefix = options?.customPrefix ? `${options.customPrefix}\n\n` : "";
      return `${prefix}# Context: ${title || "Webpage Document"}
Source: ${url}
Format: ${format.toUpperCase()}

\`\`\`${codeLang}
${content}
\`\`\`

Please use the reference context above to answer all subsequent questions accurately.`;
    }

    case "markdown_link":
      return `[${title || url}](${url})`;

    case "url_only":
      return url;

    case "raw":
    default:
      return content;
  }
}

/**
 * Copies plain text to the system clipboard using navigator.clipboard or document.execCommand fallback.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (!text) {
    return false;
  }

  // Modern navigator.clipboard API
  if (typeof navigator !== "undefined" && navigator.clipboard && navigator.clipboard.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch (err) {
      console.warn("navigator.clipboard.writeText failed, trying fallback:", err);
    }
  }

  // Fallback: temporary textarea in DOM
  if (typeof document !== "undefined" && document.createElement) {
    try {
      const textarea = document.createElement("textarea");
      textarea.value = text;
      textarea.style.position = "fixed";
      textarea.style.left = "-9999px";
      textarea.style.top = "-9999px";
      document.body.appendChild(textarea);
      textarea.focus();
      textarea.select();
      const successful = document.execCommand("copy");
      document.body.removeChild(textarea);
      return successful;
    } catch (err) {
      console.warn("document.execCommand fallback copy failed:", err);
    }
  }

  return false;
}
