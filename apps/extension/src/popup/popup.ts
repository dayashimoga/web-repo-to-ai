// Popup controller for web-repo-to-ai browser extension

import {
  BackendStatus,
  DEFAULT_CONFIG,
  DetectedArchetype,
  ExtensionConfig,
  ExtensionResponse,
  OutputFormat,
  SourceClassification,
  TabInfo,
} from "../types/index";

export interface PopupElements {
  statusDot: HTMLDivElement;
  btnOpenOptions: HTMLButtonElement;
  archetypeBadge: HTMLSpanElement;
  tabTitle: HTMLDivElement;
  tabUrl: HTMLDivElement;
  formatSelect: HTMLSelectElement;
  budgetChips: HTMLDivElement;
  btnConvert: HTMLButtonElement;
  btnConvertText: HTMLSpanElement;
  spinner: HTMLSpanElement;
  statusMessage: HTMLDivElement;
  linkOpenWorkspace: HTMLAnchorElement;
  apiVersion: HTMLSpanElement;
}

export function getPopupElements(): PopupElements | null {
  if (typeof document === "undefined") {
    return null;
  }

  const el = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

  return {
    statusDot: el<HTMLDivElement>("statusDot"),
    btnOpenOptions: el<HTMLButtonElement>("btnOpenOptions"),
    archetypeBadge: el<HTMLSpanElement>("archetypeBadge"),
    tabTitle: el<HTMLDivElement>("tabTitle"),
    tabUrl: el<HTMLDivElement>("tabUrl"),
    formatSelect: el<HTMLSelectElement>("formatSelect"),
    budgetChips: el<HTMLDivElement>("budgetChips"),
    btnConvert: el<HTMLButtonElement>("btnConvert"),
    btnConvertText: el<HTMLSpanElement>("btnConvertText"),
    spinner: el<HTMLSpanElement>("spinner"),
    statusMessage: el<HTMLDivElement>("statusMessage"),
    linkOpenWorkspace: el<HTMLAnchorElement>("linkOpenWorkspace"),
    apiVersion: el<HTMLSpanElement>("apiVersion"),
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

  const { handleMessage } = await import("../background/index");
  return (await handleMessage(msg)) as ExtensionResponse<T>;
}

export class PopupController {
  private elements: PopupElements;
  private currentTab: TabInfo | null = null;
  private currentConfig: ExtensionConfig = { ...DEFAULT_CONFIG };
  private selectedBudget: number | null = 32000;
  private isConverting = false;

  constructor(elements: PopupElements) {
    this.elements = elements;
  }

  public async init(): Promise<void> {
    this.bindEvents();
    await this.loadInitialState();
  }

  private bindEvents(): void {
    // Open Options page
    this.elements.btnOpenOptions.addEventListener("click", () => {
      if (typeof chrome !== "undefined" && chrome.runtime && chrome.runtime.openOptionsPage) {
        chrome.runtime.openOptionsPage();
      } else if (typeof window !== "undefined") {
        window.open("../options/index.html", "_blank");
      }
    });

    // Open Web UI
    this.elements.linkOpenWorkspace.addEventListener("click", (e) => {
      e.preventDefault();
      const targetUrl = this.currentConfig.apiUrl || "http://localhost:3000";
      if (typeof chrome !== "undefined" && chrome.tabs && chrome.tabs.create) {
        chrome.tabs.create({ url: targetUrl });
      } else if (typeof window !== "undefined") {
        window.open(targetUrl, "_blank");
      }
    });

    // Budget chip clicks
    const chips = this.elements.budgetChips.querySelectorAll<HTMLButtonElement>(".chip");
    chips.forEach((chip) => {
      chip.addEventListener("click", () => {
        chips.forEach((c) => c.classList.remove("active"));
        chip.classList.add("active");
        const val = parseInt(chip.getAttribute("data-budget") || "0", 10);
        this.selectedBudget = val === 0 ? null : val;
      });
    });

    // Convert button click
    this.elements.btnConvert.addEventListener("click", () => {
      this.handleConvert();
    });
  }

  public async loadInitialState(): Promise<void> {
    // 1. Fetch config
    const configRes = await sendBackgroundMessage<ExtensionConfig>({ type: "GET_CONFIG" });
    if (configRes.success && configRes.data) {
      this.currentConfig = configRes.data;
      this.elements.formatSelect.value = this.currentConfig.defaultFormat || "markdown";
      this.syncBudgetChips(this.currentConfig.defaultTokenBudget);
    }

    // 2. Check Backend Status in parallel
    sendBackgroundMessage<BackendStatus>({ type: "CHECK_BACKEND_STATUS" }).then((res) => {
      if (res.success && res.data.connected) {
        this.elements.statusDot.className = "status-dot healthy";
        this.elements.statusDot.title = `API Connected (${res.data.latencyMs}ms)`;
        if (res.data.version) {
          this.elements.apiVersion.textContent = `v${res.data.version}`;
        }
      } else {
        this.elements.statusDot.className = "status-dot offline";
        const err = (res.success && res.data.error) || (!res.success && res.error) || "Offline";
        this.elements.statusDot.title = `API Offline: ${err}`;
      }
    });

    // 3. Query active tab
    const tabRes = await sendBackgroundMessage<TabInfo>({ type: "GET_ACTIVE_TAB" });
    if (tabRes.success && tabRes.data) {
      this.currentTab = tabRes.data;
      this.elements.tabTitle.textContent = this.currentTab.title || "Untitled Tab";
      this.elements.tabTitle.title = this.currentTab.title || "";
      this.elements.tabUrl.textContent = this.currentTab.url || "";
      this.elements.tabUrl.title = this.currentTab.url || "";

      // 4. Detect Archetype
      const detectRes = await sendBackgroundMessage<SourceClassification>({
        type: "DETECT_URL_TYPE",
        url: this.currentTab.url,
      });

      if (detectRes.success && detectRes.data) {
        this.updateArchetypeBadge(detectRes.data);
      }
    } else {
      this.elements.tabTitle.textContent = "No active tab detected";
      this.elements.tabUrl.textContent = "";
      this.elements.btnConvert.disabled = true;
    }
  }

  private syncBudgetChips(budget: number | null | undefined): void {
    const chips = this.elements.budgetChips.querySelectorAll<HTMLButtonElement>(".chip");
    chips.forEach((c) => c.classList.remove("active"));

    const targetVal = budget === null || budget === undefined ? 0 : budget;
    this.selectedBudget = targetVal === 0 ? null : targetVal;

    let matched = false;
    chips.forEach((chip) => {
      const chipBudget = parseInt(chip.getAttribute("data-budget") || "0", 10);
      if (chipBudget === targetVal) {
        chip.classList.add("active");
        matched = true;
      }
    });

    if (!matched && chips.length > 1) {
      chips[1].classList.add("active"); // default to 32k
    }
  }

  public updateArchetypeBadge(classification: SourceClassification): void {
    const badge = this.elements.archetypeBadge;
    badge.textContent = classification.displayName;

    switch (classification.archetype) {
      case "github_repository":
        badge.className = "badge badge-github";
        break;
      case "web_documentation":
        badge.className = "badge badge-docs";
        break;
      case "web_article":
        badge.className = "badge badge-article";
        break;
      case "generic_web":
      case "file_resource":
        badge.className = "badge badge-web";
        break;
      default:
        badge.className = "badge badge-unsupported";
        break;
    }

    if (!classification.isSupported) {
      this.elements.btnConvert.disabled = true;
      this.elements.statusMessage.className = "status-message error";
      this.elements.statusMessage.textContent = "Cannot convert internal browser page";
    } else {
      this.elements.btnConvert.disabled = false;
      this.elements.statusMessage.textContent = "";
    }
  }

  public async handleConvert(): Promise<void> {
    if (!this.currentTab || !this.currentTab.url || this.isConverting) {
      return;
    }

    this.isConverting = true;
    this.elements.btnConvert.disabled = true;
    this.elements.btnConvert.classList.add("converting");
    this.elements.btnConvertText.textContent = "Converting...";
    this.elements.statusMessage.className = "status-message";
    this.elements.statusMessage.textContent = "Contacting web-repo-to-ai instance...";

    try {
      const format = this.elements.formatSelect.value as OutputFormat;
      const res = await sendBackgroundMessage({
        type: "START_CONVERSION",
        url: this.currentTab.url,
        format,
        tokenBudget: this.selectedBudget,
        model: this.currentConfig.defaultTargetModel,
      });

      if (res.success) {
        this.elements.statusMessage.className = "status-message success";
        this.elements.statusMessage.textContent = "Conversion initiated successfully!";
        this.elements.btnConvertText.textContent = "Converted!";
      } else {
        this.elements.statusMessage.className = "status-message error";
        this.elements.statusMessage.textContent = `Conversion failed: ${res.error}`;
        this.elements.btnConvertText.textContent = "Retry Conversion";
        this.elements.btnConvert.disabled = false;
      }
    } catch (err: any) {
      this.elements.statusMessage.className = "status-message error";
      this.elements.statusMessage.textContent = err.message || "Failed to initiate conversion";
      this.elements.btnConvertText.textContent = "Retry Conversion";
      this.elements.btnConvert.disabled = false;
    } finally {
      this.isConverting = false;
      this.elements.btnConvert.classList.remove("converting");
    }
  }
}

// Auto-initialize when running in DOM
if (typeof document !== "undefined") {
  document.addEventListener("DOMContentLoaded", () => {
    const elements = getPopupElements();
    if (elements) {
      const controller = new PopupController(elements);
      controller.init();
    }
  });
}
