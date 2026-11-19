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
      pageSummaryCard: { style: { display: "block" } } as any,
      archetypeBadge: { textContent: "", className: "" } as any,
      tabTitle: { textContent: "", title: "" } as any,
      tabUrl: { textContent: "", title: "" } as any,
      conversionOptionsCard: { style: { display: "block" } } as any,
      formatSelect: { value: "markdown" } as any,
      budgetChips: {
        querySelectorAll: vi.fn().mockReturnValue(chipMockList),
      } as any,
      actionSection: { style: { display: "block" } } as any,
      btnConvert: {
        disabled: false,
        classList: { add: vi.fn(), remove: vi.fn() },
        addEventListener: vi.fn(),
      } as any,
      btnConvertText: { textContent: "" } as any,
      spinner: {} as any,
      statusMessage: { className: "", textContent: "" } as any,
      progressContainer: { style: { display: "none" } } as any,
      progressBar: { style: { width: "0%" } } as any,
      progressPhaseText: { textContent: "" } as any,
      resultContainer: { style: { display: "none" } } as any,
      resTokens: { textContent: "0" } as any,
      resChars: { textContent: "0" } as any,
      resFormat: { textContent: "" } as any,
      btnCopyResult: { textContent: "📋 Copy Content", addEventListener: vi.fn() } as any,
      btnCopyPrompt: { textContent: "✨ Copy as AI Prompt", addEventListener: vi.fn() } as any,
      btnDownloadResult: { addEventListener: vi.fn() } as any,
      btnOpenInWorkspace: { addEventListener: vi.fn() } as any,
      btnResetConversion: { addEventListener: vi.fn() } as any,
      historyCard: { style: { display: "block" } } as any,
      historyToggle: { addEventListener: vi.fn() } as any,
      historyCount: { textContent: "0" } as any,
      historyArrow: { textContent: "▶" } as any,
      historyList: {
        style: { display: "none" },
        innerHTML: "",
        appendChild: vi.fn(),
      } as any,
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

  describe("updateProgress & showResult & resetToIdle", () => {
    it("updates progress bar and phase text", () => {
      controller.updateProgress({ pct: 60, phaseText: "Optimizing tokens..." });
      expect(elements.progressContainer.style.display).toBe("block");
      expect(elements.progressBar.style.width).toBe("60%");
      expect(elements.progressPhaseText.textContent).toBe("Optimizing tokens...");
    });

    it("displays conversion result and hides form controls", () => {
      controller.showResult({
        jobId: "job-popup-1",
        url: "https://github.com/rust-lang/rust",
        format: "markdown",
        content: "# Rust Readme",
        tokenCount: 1540,
        charCount: 6160,
        durationMs: 850,
        archetype: "github_repository",
      });

      expect(elements.resultContainer.style.display).toBe("block");
      expect(elements.conversionOptionsCard.style.display).toBe("none");
      expect(elements.actionSection.style.display).toBe("none");
      expect(elements.resTokens.textContent).toBe("1,540");
      expect(elements.resChars.textContent).toBe("6.0 KB");
      expect(elements.resFormat.textContent).toBe("MARKDOWN");
    });

    it("resets back to idle state for another conversion", () => {
      controller.resetToIdle();
      expect(elements.resultContainer.style.display).toBe("none");
      expect(elements.conversionOptionsCard.style.display).toBe("block");
      expect(elements.actionSection.style.display).toBe("block");
      expect(elements.btnConvert.disabled).toBe(false);
      expect(elements.btnConvertText.textContent).toBe("Convert Current Tab");
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
    it("executes conversion pipeline and presents result view upon success", async () => {
      // 1. Initial State
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        statusText: "OK",
        json: async () => ({ status: "healthy", version: "0.1.0" }),
      });
      await controller.loadInitialState();

      // 2. Conversion mock via fetch
      globalThis.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          ok: true,
          status: 202,
          json: async () => ({
            job_id: "job-pop-success",
            status: "queued",
            sources: ["https://github.com/rust-lang/rust"],
          }),
        })
        .mockResolvedValueOnce({
          ok: true,
          status: 200,
          json: async () => ({
            job_id: "job-pop-success",
            status: "completed",
          }),
        })
        .mockResolvedValueOnce({
          ok: true,
          status: 200,
          text: async () => "# Converted Output Content",
        });

      elements.formatSelect.value = "markdown";
      await controller.handleConvert();

      expect(elements.resultContainer.style.display).toBe("block");
      expect(elements.resFormat.textContent).toBe("MARKDOWN");
      expect(elements.conversionOptionsCard.style.display).toBe("none");
    });
  });

  describe("refreshHistory", () => {
    it("updates historyCount and renders empty message when no items exist", async () => {
      await controller.refreshHistory();
      expect(elements.historyCount.textContent).toBe("0");
      expect(elements.historyList.innerHTML).toContain("No recent conversions yet");
    });
  });
});
