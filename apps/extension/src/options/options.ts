// Options page controller for web-repo-to-ai browser extension

import { sanitizeError } from "../security/redaction";
import {
  BackendStatus,
  DEFAULT_CONFIG,
  ExtensionConfig,
  ExtensionResponse,
  OutputFormat,
  TargetModel,
} from "../types/index";

// Form DOM element references
interface OptionsFormElements {
  apiUrl: HTMLInputElement;
  apiKey: HTMLInputElement;
  btnToggleKey: HTMLButtonElement;
  btnTestConnection: HTMLButtonElement;
  connectionStatus: HTMLDivElement;
  defaultFormat: HTMLSelectElement;
  defaultTokenBudget: HTMLInputElement;
  defaultTargetModel: HTMLSelectElement;
  autoDetectSource: HTMLInputElement;
  includeFrontmatter: HTMLInputElement;
  sendDomSnapshot: HTMLInputElement;
  btnSave: HTMLButtonElement;
  btnReset: HTMLButtonElement;
  toast: HTMLDivElement;
}

/**
 * Resolves form elements from the DOM.
 */
export function getFormElements(): OptionsFormElements | null {
  if (typeof document === "undefined") {
    return null;
  }

  const el = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

  return {
    apiUrl: el<HTMLInputElement>("apiUrl"),
    apiKey: el<HTMLInputElement>("apiKey"),
    btnToggleKey: el<HTMLButtonElement>("btnToggleKey"),
    btnTestConnection: el<HTMLButtonElement>("btnTestConnection"),
    connectionStatus: el<HTMLDivElement>("connectionStatus"),
    defaultFormat: el<HTMLSelectElement>("defaultFormat"),
    defaultTokenBudget: el<HTMLInputElement>("defaultTokenBudget"),
    defaultTargetModel: el<HTMLSelectElement>("defaultTargetModel"),
    autoDetectSource: el<HTMLInputElement>("autoDetectSource"),
    includeFrontmatter: el<HTMLInputElement>("includeFrontmatter"),
    sendDomSnapshot: el<HTMLInputElement>("sendDomSnapshot"),
    btnSave: el<HTMLButtonElement>("btnSave"),
    btnReset: el<HTMLButtonElement>("btnReset"),
    toast: el<HTMLDivElement>("toast"),
  };
}

/**
 * Sends a message to the background service worker or calls mock/fallback.
 */
async function sendBackgroundMessage<T>(msg: any): Promise<ExtensionResponse<T>> {
  if (typeof chrome !== "undefined" && chrome.runtime && chrome.runtime.sendMessage) {
    return new Promise((resolve) => {
      chrome.runtime.sendMessage(msg, (response) => {
        if (chrome.runtime.lastError) {
          resolve({
            success: false,
            error: chrome.runtime.lastError.message || "Failed to communicate with service worker",
          });
        } else {
          resolve(response || { success: false, error: "Empty response from service worker" });
        }
      });
    });
  }

  // Fallback direct execution for headless testing
  const { handleMessage } = await import("../background/index");
  return (await handleMessage(msg)) as ExtensionResponse<T>;
}

/**
 * Populates form fields from a configuration object.
 */
export function populateForm(elements: OptionsFormElements, config: ExtensionConfig): void {
  elements.apiUrl.value = config.apiUrl || DEFAULT_CONFIG.apiUrl;
  elements.apiKey.value = config.apiKey || "";
  elements.defaultFormat.value = config.defaultFormat || DEFAULT_CONFIG.defaultFormat;
  elements.defaultTokenBudget.value =
    config.defaultTokenBudget !== null && config.defaultTokenBudget !== undefined
      ? String(config.defaultTokenBudget)
      : "";
  elements.defaultTargetModel.value =
    config.defaultTargetModel || DEFAULT_CONFIG.defaultTargetModel;
  elements.autoDetectSource.checked = config.autoDetectSource !== false;
  elements.includeFrontmatter.checked = config.includeFrontmatter !== false;
  elements.sendDomSnapshot.checked = config.sendDomSnapshot === true;
}

/**
 * Extracts configuration values from form fields.
 */
export function extractFormValues(elements: OptionsFormElements): Partial<ExtensionConfig> {
  const rawBudget = elements.defaultTokenBudget.value.trim();
  const tokenBudget = rawBudget ? parseInt(rawBudget, 10) : null;

  return {
    apiUrl: elements.apiUrl.value.trim(),
    apiKey: elements.apiKey.value.trim(),
    defaultFormat: elements.defaultFormat.value as OutputFormat,
    defaultTokenBudget: tokenBudget,
    defaultTargetModel: elements.defaultTargetModel.value as TargetModel,
    autoDetectSource: elements.autoDetectSource.checked,
    includeFrontmatter: elements.includeFrontmatter.checked,
    sendDomSnapshot: elements.sendDomSnapshot.checked,
  };
}

/**
 * Displays a temporary toast notification.
 */
export function showToast(toast: HTMLDivElement, message: string, isError = false): void {
  toast.textContent = message;
  toast.style.backgroundColor = isError ? "var(--danger)" : "var(--success)";
  toast.classList.add("show");
  setTimeout(() => {
    toast.classList.remove("show");
  }, 2500);
}

/**
 * Executes a backend connection probe and updates UI status indicator.
 */
export async function testConnection(elements: OptionsFormElements): Promise<BackendStatus> {
  elements.connectionStatus.style.display = "inline-flex";
  elements.connectionStatus.className = "status-badge testing";
  elements.connectionStatus.textContent = "Testing connection...";
  elements.btnTestConnection.disabled = true;

  try {
    const res = await sendBackgroundMessage<BackendStatus>({
      type: "CHECK_BACKEND_STATUS",
      apiUrl: elements.apiUrl.value.trim(),
      apiKey: elements.apiKey.value.trim(),
    });

    if (res.success && res.data.connected) {
      elements.connectionStatus.className = "status-badge healthy";
      elements.connectionStatus.textContent = `Connected (${res.data.latencyMs}ms) - API healthy`;
      return res.data;
    } else {
      elements.connectionStatus.className = "status-badge offline";
      const errMsg = sanitizeError(
        (res.success && res.data.error) || (!res.success && res.error) || "Offline"
      );
      elements.connectionStatus.textContent = `Offline: ${errMsg}`;
      return res.success ? res.data : { connected: false, error: res.error };
    }
  } finally {
    elements.btnTestConnection.disabled = false;
  }
}

/**
 * Initializes the options page event listeners and initial load.
 */
export async function initOptions(): Promise<void> {
  const elements = getFormElements();
  if (!elements) return;

  // Load configuration from background
  const res = await sendBackgroundMessage<ExtensionConfig>({ type: "GET_CONFIG" });
  if (res.success && res.data) {
    populateForm(elements, res.data);
  } else {
    populateForm(elements, DEFAULT_CONFIG);
  }

  // Toggle API key visibility
  elements.btnToggleKey.addEventListener("click", () => {
    if (elements.apiKey.type === "password") {
      elements.apiKey.type = "text";
      elements.btnToggleKey.textContent = "Hide";
    } else {
      elements.apiKey.type = "password";
      elements.btnToggleKey.textContent = "Show";
    }
  });

  // Test connection button
  elements.btnTestConnection.addEventListener("click", () => {
    testConnection(elements);
  });

  // Save settings button
  elements.btnSave.addEventListener("click", async () => {
    const values = extractFormValues(elements);
    elements.btnSave.disabled = true;

    try {
      const saveRes = await sendBackgroundMessage<ExtensionConfig>({
        type: "SAVE_CONFIG",
        config: values,
      });

      if (saveRes.success) {
        showToast(elements.toast, "Settings saved successfully!");
      } else {
        showToast(elements.toast, `Error: ${saveRes.error}`, true);
      }
    } catch (err: any) {
      showToast(elements.toast, `Failed to save: ${err.message}`, true);
    } finally {
      elements.btnSave.disabled = false;
    }
  });

  // Reset defaults button
  elements.btnReset.addEventListener("click", async () => {
    const confirmed =
      typeof window === "undefined" ||
      !window.confirm ||
      window.confirm("Reset all settings to default values?");

    if (confirmed) {
      const resetRes = await sendBackgroundMessage<ExtensionConfig>({ type: "RESET_CONFIG" });
      if (resetRes.success && resetRes.data) {
        populateForm(elements, resetRes.data);
        showToast(elements.toast, "Settings reset to defaults");
      }
    }
  });
}

// Auto-initialize when running in DOM
if (typeof document !== "undefined") {
  document.addEventListener("DOMContentLoaded", () => {
    initOptions();
  });
}
