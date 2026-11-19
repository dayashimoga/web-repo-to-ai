import { beforeEach, describe, expect, it, vi } from "vitest";
import { PopupController, PopupElements } from "../src/popup/popup";
import { setupMockChrome } from "./mocks/chrome";

describe("Popup Controller (apps/extension)", () => {
  let mockChrome: ReturnType<typeof setupMockChrome>;
  let elements: PopupElements;
  let controller: PopupController;

  beforeEach(() => {
    mockChrome = setupMockChrome();

    // Create chip mock elements
    const chipMockList = [
      {
        getAttribute: (attr: string) => (attr === "data-budget" ? "8000" : null),
        classList: { add: vi.fn(), remove: vi.fn() },
        addEventListener: vi.fn(),
      },
      {
        getAttribute: (attr: string) => (attr === "data-budget" ? "32000" : null),
        classList: { add: vi.fn(), remove: vi.fn() },
        addEventListener: vi.fn(),
      },
      {
        getAttribute: (attr: string) => (attr === "data-budget" ? "64000" : null),
        classList: { add: vi.fn(), remove: vi.fn() },
        addEventListener: vi.fn(),
      },
      {
        getAttribute: (attr: string) => (attr === "data-budget" ? "0" : null),
        classList: { add: vi.fn(), remove: vi.fn() },
        addEventListener: vi.fn(),
      },
    ];

    elements = {
      statusDot: { className: "", title: "" } as any,
      btnOpenOptions: { addEventListener: vi.fn() } as any,
      archetypeBadge: { textContent: "", className: "" } as any,
      tabTitle: { textContent: "", title: "" } as any,
      tabUrl: { textContent: "", title: "" } as any,
      formatSelect: { value: "markdown" } as any,
      budgetChips: {
        querySelectorAll: vi.fn().mockReturnValue(chipMockList),
      } as any,
      btnConvert: {
        disabled: false,
        classList: { add: vi.fn(), remove: vi.fn() },
        addEventListener: vi.fn(),
      } as any,
      btnConvertText: { textContent: "" } as any,
      spinner: {} as any,
      statusMessage: { className: "", textContent: "" } as any,
      linkOpenWorkspace: { addEventListener: vi.fn() } as any,
      apiVersion: { textContent: "" } as any,
    };

    controller = new PopupController(elements);
  });

  describe("updateArchetypeBadge", () => {
    it("updates badge for GitHub repository and enables convert button", () => {
      controller.updateArchetypeBadge({
        archetype: "github_repository",
        displayName: "GitHub: rust-lang/rust",
        url: "https://github.com/rust-lang/rust",
        isSupported: true,
      });

      expect(elements.archetypeBadge.textContent).toBe("GitHub: rust-lang/rust");
      expect(elements.archetypeBadge.className).toBe("badge badge-github");
      expect(elements.btnConvert.disabled).toBe(false);
      expect(elements.statusMessage.textContent).toBe("");
    });

    it("updates badge for Documentation and enables convert button", () => {
      controller.updateArchetypeBadge({
        archetype: "web_documentation",
        displayName: "Technical Documentation",
        url: "https://docs.rs/tokio",
        isSupported: true,
      });

      expect(elements.archetypeBadge.className).toBe("badge badge-docs");
      expect(elements.btnConvert.disabled).toBe(false);
    });

    it("disables convert button and warns for unsupported pages", () => {
      controller.updateArchetypeBadge({
        archetype: "unknown",
        displayName: "Internal Browser Page",
        url: "chrome://extensions",
        isSupported: false,
      });

      expect(elements.archetypeBadge.className).toBe("badge badge-unsupported");
      expect(elements.btnConvert.disabled).toBe(true);
      expect(elements.statusMessage.textContent).toMatch(/Cannot convert/i);
    });
  });

  describe("loadInitialState", () => {
    it("loads configuration, tab information, and detects URL type", async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        statusText: "OK",
        json: async () => ({ status: "healthy", version: "0.1.0" }),
      });

      await controller.loadInitialState();

      expect(elements.tabTitle.textContent).toMatch(/rust-lang\/rust/i);
      expect(elements.tabUrl.textContent).toBe("https://github.com/rust-lang/rust");
      expect(elements.archetypeBadge.textContent).toBe("GitHub: rust-lang/rust");
      expect(elements.archetypeBadge.className).toBe("badge badge-github");
    });
  });

  describe("handleConvert", () => {
    it("executes conversion pipeline when valid tab is loaded", async () => {
      // Setup initial state first
      await controller.loadInitialState();

      elements.formatSelect.value = "markdown";
      await controller.handleConvert();

      expect(elements.statusMessage.className).toBe("status-message success");
      expect(elements.statusMessage.textContent).toMatch(/Conversion initiated successfully/i);
      expect(elements.btnConvertText.textContent).toBe("Converted!");
    });
  });
});
