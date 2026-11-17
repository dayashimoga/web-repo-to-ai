/**
 * High-performance, zero-dependency Syntax Highlighter & HTML Tokenizer
 *
 * Implements:
 * 1. Safe HTML escaping guaranteeing zero XSS vulnerability.
 * 2. Token classification for JSON, XML, Markdown, Rust, TOML, and Plaintext.
 * 3. Line-by-line formatting with accessible line number gutter support.
 */

export function escapeHtml(unsafe: string): string {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export interface HighlightedLine {
  lineNumber: number;
  html: string;
}

/**
 * Tokenizes and highlights JSON content safely.
 */
function highlightJson(code: string): string {
  const jsonRegex =
    /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+-]?\d+)?|[{}[\],:])/g;

  let lastIndex = 0;
  let result = "";
  let match: RegExpExecArray | null;

  while ((match = jsonRegex.exec(code)) !== null) {
    if (match.index > lastIndex) {
      result += escapeHtml(code.slice(lastIndex, match.index));
    }
    const token = match[0];
    if (/^"/.test(token)) {
      if (/:$/.test(token)) {
        const keyPart = token.slice(0, -1);
        result += `<span class="text-brand-400 font-semibold">${escapeHtml(keyPart)}</span>:`;
      } else {
        result += `<span class="text-green-400">${escapeHtml(token)}</span>`;
      }
    } else if (/true|false/.test(token)) {
      result += `<span class="text-accent-400 font-bold">${escapeHtml(token)}</span>`;
    } else if (/null/.test(token)) {
      result += `<span class="text-red-400 italic">${escapeHtml(token)}</span>`;
    } else if (/[{}[\],:]/.test(token)) {
      result += `<span class="text-surface-400 font-bold">${escapeHtml(token)}</span>`;
    } else {
      result += `<span class="text-amber-400 font-mono">${escapeHtml(token)}</span>`;
    }
    lastIndex = jsonRegex.lastIndex;
  }

  if (lastIndex < code.length) {
    result += escapeHtml(code.slice(lastIndex));
  }

  return result;
}

/**
 * Tokenizes and highlights XML content safely.
 */
function highlightXml(code: string): string {
  const xmlRegex =
    /(<!--[\s\S]*?-->|<\?[^?]*\?>|<\/?[a-zA-Z0-9_:-]+|\/?>|[a-zA-Z_:-]+(?==)|"[^"]*")/g;

  let lastIndex = 0;
  let result = "";
  let match: RegExpExecArray | null;

  while ((match = xmlRegex.exec(code)) !== null) {
    if (match.index > lastIndex) {
      result += escapeHtml(code.slice(lastIndex, match.index));
    }
    const token = match[0];
    if (token.startsWith("<!--")) {
      result += `<span class="text-surface-500 italic">${escapeHtml(token)}</span>`;
    } else if (token.startsWith("<?")) {
      result += `<span class="text-surface-400 italic">${escapeHtml(token)}</span>`;
    } else if (token.startsWith("</") || token.startsWith("<")) {
      result += `<span class="text-brand-400 font-semibold">${escapeHtml(token)}</span>`;
    } else if (token === ">" || token === "/>") {
      result += `<span class="text-brand-400 font-semibold">${escapeHtml(token)}</span>`;
    } else if (token.startsWith('"')) {
      result += `<span class="text-green-400">${escapeHtml(token)}</span>`;
    } else {
      result += `<span class="text-accent-400">${escapeHtml(token)}</span>`;
    }
    lastIndex = xmlRegex.lastIndex;
  }

  if (lastIndex < code.length) {
    result += escapeHtml(code.slice(lastIndex));
  }

  return result;
}

/**
 * Tokenizes and highlights Markdown line content.
 */
function highlightMarkdownLine(line: string): string {
  // Headers (#, ##, ###)
  if (/^#{1,6}\s/.test(line)) {
    return `<span class="text-brand-400 font-bold">${escapeHtml(line)}</span>`;
  }
  // Blockquotes
  if (/^>\s/.test(line)) {
    return `<span class="text-surface-400 italic">${escapeHtml(line)}</span>`;
  }
  // Fenced code block markers
  if (/^```/.test(line)) {
    return `<span class="text-accent-400 font-mono font-bold">${escapeHtml(line)}</span>`;
  }
  // Horizontal rules
  if (/^---$|^\*\*\*$|^___$/.test(line.trim())) {
    return `<span class="text-surface-600 font-bold">${escapeHtml(line)}</span>`;
  }
  // Unordered list items
  if (/^(\s*)[-*+]\s/.test(line)) {
    return line.replace(/^(\s*[-*+]\s)(.*)/, (_, bullet, rest) => {
      return `<span class="text-brand-400 font-bold">${escapeHtml(bullet)}</span>${highlightInlineMarkdown(rest)}`;
    });
  }
  // Ordered list items
  if (/^(\s*)\d+\.\s/.test(line)) {
    return line.replace(/^(\s*\d+\.\s)(.*)/, (_, num, rest) => {
      return `<span class="text-accent-400 font-bold">${escapeHtml(num)}</span>${highlightInlineMarkdown(rest)}`;
    });
  }

  return highlightInlineMarkdown(line);
}

function highlightInlineMarkdown(text: string): string {
  // Extract backtick code spans before escaping the rest
  const parts = text.split(/(`[^`]+`)/g);
  return parts
    .map((part) => {
      if (part.startsWith("`") && part.endsWith("`") && part.length > 2) {
        const codeInner = part.slice(1, -1);
        return `<span class="text-accent-300 font-mono bg-surface-800/70 px-1 py-0.5 rounded text-[11px]">${escapeHtml(codeInner)}</span>`;
      }
      return escapeHtml(part);
    })
    .join("");
}

/**
 * Tokenizes and highlights code in general programming languages (Rust, TOML, TS, etc.).
 */
function highlightCodeLine(line: string, language: string): string {
  // Comment check
  if (/^\s*(\/\/|#)/.test(line)) {
    return `<span class="text-surface-500 italic">${escapeHtml(line)}</span>`;
  }

  // Keywords set based on language
  let keywords: RegExp;
  if (language === "rust") {
    keywords =
      /\b(pub|fn|struct|enum|impl|mod|use|let|mut|match|if|else|return|trait|type|where|async|await|const|static|for|while|loop|break|continue|as|in|ref|crate|self|Self|true|false|Ok|Err|Some|None)\b/g;
  } else if (language === "toml") {
    keywords = /\b(package|workspace|members|dependencies|version|name|edition|license)\b/g;
  } else {
    keywords =
      /\b(const|let|var|function|return|import|export|from|class|extends|if|else|for|while|try|catch|new|async|await|default)\b/g;
  }

  const safe = escapeHtml(line);
  return safe
    .replace(keywords, '<span class="text-accent-400 font-semibold">$1</span>')
    .replace(/(".*?")/g, '<span class="text-green-400">$1</span>')
    .replace(/\b(\d+)\b/g, '<span class="text-amber-400 font-mono">$1</span>');
}

/**
 * Main syntax highlighter dispatcher.
 */
export function highlightSyntax(code: string, language: string): string {
  if (!code) return "";
  const lang = language.toLowerCase();

  if (lang === "json" || lang === "jsonl") {
    return highlightJson(code);
  }

  if (lang === "xml" || lang === "html") {
    return highlightXml(code);
  }

  const lines = code.split("\n");
  if (lang === "markdown" || lang === "md") {
    return lines.map((l) => highlightMarkdownLine(l)).join("\n");
  }

  return lines.map((l) => highlightCodeLine(l, lang)).join("\n");
}

/**
 * Splits formatted code into numbered lines for accessible code presentation.
 */
export function highlightLines(code: string, language: string): HighlightedLine[] {
  if (!code) return [];
  const lang = language.toLowerCase();

  if (lang === "json" || lang === "jsonl") {
    const rawLines = code.split("\n");
    return rawLines.map((line, idx) => ({
      lineNumber: idx + 1,
      html: highlightJson(line),
    }));
  }

  if (lang === "xml" || lang === "html") {
    const rawLines = code.split("\n");
    return rawLines.map((line, idx) => ({
      lineNumber: idx + 1,
      html: highlightXml(line),
    }));
  }

  const rawLines = code.split("\n");
  return rawLines.map((line, idx) => {
    let lineHtml: string;
    if (lang === "markdown" || lang === "md") {
      lineHtml = highlightMarkdownLine(line);
    } else {
      lineHtml = highlightCodeLine(line, lang);
    }
    return {
      lineNumber: idx + 1,
      html: lineHtml || "&nbsp;",
    };
  });
}
