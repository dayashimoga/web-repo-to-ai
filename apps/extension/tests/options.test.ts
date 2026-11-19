import { beforeEach, describe, expect, it, vi } from "vitest";
import { extractFormValues, populateForm, showToast, testConnection } from "../src/options/options";
import { DEFAULT_CONFIG, ExtensionConfig } from "../src/types/index";
import { setupMockChrome } from "./mocks/chrome";

describe("Options Page Controller (apps/extension)", () => {
  let mockChrome: ReturnType<typeof setupMockChrome>;

  // Minimal simulated form elements
  let formElements: any;

  beforeEach(() => {
    mockChrome = setupMockChrome();

    formElements = {
      apiUrl: { value: "http://localhost:3000" },
      apiKey: { value: "", type: "password" },
      btnToggleKey: { textContent: "Show" },
      btnTestConnection: { disabled: false },
      connectionStatus: {
        style: { display: "none" },
        className: "",
        textContent: "",
      },
      defaultFormat: { value: "markdown" },
      defaultTokenBudget: { value: "32000" },
      defaultTargetModel: { value: "gpt-4o" },
      autoDetectSource: { checked: true },
      includeFrontmatter: { checked: true },
      sendDomSnapshot: { checked: false },
      btnSave: { disabled: false },
      btnReset: { disabled: false },
      toast: {
        textContent: "",
        style: { backgroundColor: "" },
        classList: {
          add: vi.fn(),
          remove: vi.fn(),
        },
      },
    };
  });

  describe("populateForm", () => {
    it("populates all input elements from ExtensionConfig", () => {
      const customConfig: ExtensionConfig = {
        apiUrl: "https://remote.ai.engine:9000",
        apiKey: "wr_live_test123",
        defaultFormat: "zip",
        defaultTokenBudget: 128000,
        defaultTargetModel: "claude-3-5-sonnet",
        autoDetectSource: false,
        includeFrontmatter: false,
        sendDomSnapshot: true,
      };

      populateForm(formElements, customConfig);

      expect(formElements.apiUrl.value).toBe("https://remote.ai.engine:9000");
      expect(formElements.apiKey.value).toBe("wr_live_test123");
      expect(formElements.defaultFormat.value).toBe("zip");
      expect(formElements.defaultTokenBudget.value).toBe("128000");
      expect(formElements.defaultTargetModel.value).toBe("claude-3-5-sonnet");
      expect(formElements.autoDetectSource.checked).toBe(false);
      expect(formElements.includeFrontmatter.checked).toBe(false);
      expect(formElements.sendDomSnapshot.checked).toBe(true);
    });

    it("handles null token budget by setting empty string", () => {
      const customConfig: ExtensionConfig = {
        ...DEFAULT_CONFIG,
        defaultTokenBudget: null,
      };

      populateForm(formElements, customConfig);
      expect(formElements.defaultTokenBudget.value).toBe("");
    });
  });

  describe("extractFormValues", () => {
    it("extracts and parses values from form elements", () => {
      formElements.apiUrl.value = "  http://api.local:8080/  ";
      formElements.apiKey.value = "  wr_live_key999  ";
      formElements.defaultFormat.value = "json";
      formElements.defaultTokenBudget.value = "64000";
      formElements.defaultTargetModel.value = "gemini-1-5-pro";
      formElements.autoDetectSource.checked = true;
      formElements.includeFrontmatter.checked = false;
      formElements.sendDomSnapshot.checked = true;

      const extracted = extractFormValues(formElements);

      expect(extracted.apiUrl).toBe("http://api.local:8080/");
      expect(extracted.apiKey).toBe("wr_live_key999");
      expect(extracted.defaultFormat).toBe("json");
      expect(extracted.defaultTokenBudget).toBe(64000);
      expect(extracted.defaultTargetModel).toBe("gemini-1-5-pro");
      expect(extracted.autoDetectSource).toBe(true);
      expect(extracted.includeFrontmatter).toBe(false);
      expect(extracted.sendDomSnapshot).toBe(true);
    });

    it("parses empty token budget as null", () => {
      formElements.defaultTokenBudget.value = "   ";
      const extracted = extractFormValues(formElements);
      expect(extracted.defaultTokenBudget).toBeNull();
    });
  });

  describe("showToast", () => {
    it("displays success toast", () => {
      showToast(formElements.toast, "Saved successfully!", false);
      expect(formElements.toast.textContent).toBe("Saved successfully!");
      expect(formElements.toast.style.backgroundColor).toBe("var(--success)");
      expect(formElements.toast.classList.add).toHaveBeenCalledWith("show");
    });

    it("displays error toast", () => {
      showToast(formElements.toast, "Save failed!", true);
      expect(formElements.toast.textContent).toBe("Save failed!");
      expect(formElements.toast.style.backgroundColor).toBe("var(--danger)");
      expect(formElements.toast.classList.add).toHaveBeenCalledWith("show");
    });
  });

  describe("testConnection", () => {
    it("handles successful backend connection probe", async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        statusText: "OK",
        json: async () => ({ status: "healthy", version: "0.1.0" }),
      });

      const result = await testConnection(formElements);
      expect(result.connected).toBe(true);
      expect(formElements.connectionStatus.className).toBe("status-badge healthy");
      expect(formElements.connectionStatus.textContent).toMatch(/Connected.*API healthy/);
      expect(formElements.btnTestConnection.disabled).toBe(false);
    });

    it("handles unreachable backend connection probe", async () => {
      globalThis.fetch = vi.fn().mockRejectedValue(new Error("Network Error"));

      const result = await testConnection(formElements);
      expect(result.connected).toBe(false);
      expect(formElements.connectionStatus.className).toBe("status-badge offline");
      expect(formElements.connectionStatus.textContent).toMatch(/Offline: Network Error/);
      expect(formElements.btnTestConnection.disabled).toBe(false);
    });
  });
});
