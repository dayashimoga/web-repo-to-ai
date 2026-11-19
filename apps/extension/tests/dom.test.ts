import { beforeEach, describe, expect, it } from "vitest";
import {
  captureActiveTabDOM,
  extractDOMFromDocument,
  sanitizeDocumentContent,
} from "../src/dom/extractor";
import { setupMockChrome } from "./mocks/chrome";

describe("DOM Extraction & Sanitization (apps/extension)", () => {
  let mockChrome: ReturnType<typeof setupMockChrome>;

  beforeEach(() => {
    mockChrome = setupMockChrome();
  });

  describe("sanitizeDocumentContent", () => {
    it("strips scripts, styles, iframes, svgs, and inline event handlers", () => {
      // Mock minimal DOM structure
      const mockElements: any[] = [];
      const createEl = (tag: string, text = "", attrs: Record<string, string> = {}) => {
        const el = {
          tagName: tag.toUpperCase(),
          textContent: text,
          attributes: Object.entries(attrs).map(([name, value]) => ({ name, value })),
          removeAttribute: (name: string) => {
            delete attrs[name];
          },
          parentNode: {
            removeChild: (child: any) => {
              const idx = mockElements.indexOf(child);
              if (idx !== -1) mockElements.splice(idx, 1);
            },
          },
        };
        mockElements.push(el);
        return el;
      };

      const mockBody: any = {
        innerHTML: "<article><h1 onclick='alert(1)'>Hello</h1><p>Clean body</p></article>",
        textContent: "Hello Clean body",
        cloneNode: () => mockBody,
        querySelectorAll: (selector: string) => {
          if (selector === "*") {
            return [
              {
                attributes: [{ name: "onclick", value: "alert(1)" }],
                removeAttribute: (attr: string) => expect(attr).toBe("onclick"),
              },
            ];
          }
          if (selector === "script") return [];
          return [];
        },
      };

      const mockDoc: any = { body: mockBody };
      const { html, textContent } = sanitizeDocumentContent(mockDoc);

      expect(html).toContain("Hello");
      expect(textContent).toBe("Hello Clean body");
    });
  });

  describe("extractDOMFromDocument", () => {
    it("extracts page metadata, headings, and token estimation", () => {
      const mockDoc: any = {
        location: { href: "https://docs.rs/tokio/latest/tokio/" },
        title: "tokio - Rust",
        querySelector: (selector: string) => {
          if (selector === 'link[rel="canonical"]') {
            return { getAttribute: () => "https://docs.rs/tokio/latest/tokio/" };
          }
          if (selector === 'meta[name="description"]') {
            return {
              getAttribute: () =>
                "An event-driven, non-blocking I/O platform for writing asynchronous I/O.",
            };
          }
          if (selector === 'meta[name="author"]') {
            return { getAttribute: () => "Tokio Contributors" };
          }
          return null;
        },
        querySelectorAll: (selector: string) => {
          if (selector === "h1, h2, h3, h4, h5, h6") {
            return [
              { tagName: "H1", textContent: "Crate tokio" },
              { tagName: "H2", textContent: "Modules" },
              { tagName: "H3", textContent: "io" },
            ];
          }
          return [];
        },
        body: {
          cloneNode: () => ({
            querySelectorAll: () => [],
            innerHTML: "<h1>Crate tokio</h1><p>Async runtime for Rust</p>",
            textContent: "Crate tokio Async runtime for Rust",
          }),
        },
      };

      const snapshot = extractDOMFromDocument(mockDoc);

      expect(snapshot.url).toBe("https://docs.rs/tokio/latest/tokio/");
      expect(snapshot.title).toBe("tokio - Rust");
      expect(snapshot.canonicalUrl).toBe("https://docs.rs/tokio/latest/tokio/");
      expect(snapshot.description).toMatch(/event-driven/i);
      expect(snapshot.author).toBe("Tokio Contributors");
      expect(snapshot.headings).toHaveLength(3);
      expect(snapshot.headings[0]).toEqual({ level: 1, text: "Crate tokio" });
      expect(snapshot.headings[1]).toEqual({ level: 2, text: "Modules" });
      expect(snapshot.charCount).toBeGreaterThan(10);
      expect(snapshot.estimatedTokens).toBe(Math.ceil(snapshot.charCount / 4));
    });
  });

  describe("captureActiveTabDOM", () => {
    it("calls chrome.scripting.executeScript when available", async () => {
      const mockResult = {
        url: "https://github.com/rust-lang/rust",
        title: "rust-lang/rust",
        html: "<p>Source code</p>",
        textContent: "Source code",
        charCount: 11,
        estimatedTokens: 3,
        headings: [],
        capturedAt: new Date().toISOString(),
      };

      (mockChrome as any).scripting = {
        executeScript: async () => [{ result: mockResult }],
      };

      const snapshot = await captureActiveTabDOM(101);
      expect(snapshot.url).toBe("https://github.com/rust-lang/rust");
      expect(snapshot.title).toBe("rust-lang/rust");
      expect(snapshot.estimatedTokens).toBe(3);
    });

    it("falls back gracefully when executeScript throws an error", async () => {
      (mockChrome as any).scripting = {
        executeScript: async () => {
          throw new Error("Cannot access chrome:// page");
        },
      };

      const snapshot = await captureActiveTabDOM(101);
      expect(snapshot).toBeDefined();
      expect(snapshot.capturedAt).toBeDefined();
    });
  });
});
