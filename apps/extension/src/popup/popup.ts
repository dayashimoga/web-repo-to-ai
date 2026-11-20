// Popup controller for web-repo-to-ai browser extension

import { copyToClipboard, formatContentForClipboard } from "../handoff/clipboard";
import { buildWorkspaceUrl, navigateToUrl } from "../handoff/deeplink";
import { sanitizeDownloadFilename, triggerFileDownload } from "../handoff/download";
import { sanitizeError } from "../security/redaction";
import {
  BackendStatus,
  ConversionProgress,
  ConversionResult,
  DEFAULT_CONFIG,
  ExtensionConfig,
  ExtensionResponse,
  HistoryItem,
  OutputFormat,
  SourceClassification,
  TabInfo,
} from "../types/index";

export interface PopupElements {
  statusDot: HTMLDivElement;
  btnOpenOptions: HTMLButtonElement;
  pageSummaryCard: HTMLElement;
  archetypeBadge: HTMLSpanElement;
  tabTitle: HTMLDivElement;
  tabUrl: HTMLDivElement;
  conversionOptionsCard: HTMLElement;
  formatSelect: HTMLSelectElement;
  budgetChips: HTMLDivElement;
  actionSection: HTMLElement;
  btnConvert: HTMLButtonElement;
  btnConvertText: HTMLSpanElement;
  spinner: HTMLSpanElement;
  statusMessage: HTMLDivElement;
  progressContainer: HTMLDivElement;
  progressBar: HTMLDivElement;
  progressPhaseText: HTMLDivElement;
  resultContainer: HTMLElement;
  resTokens: HTMLSpanElement;
  resChars: HTMLSpanElement;
  resFormat: HTMLSpanElement;
  btnCopyResult: HTMLButtonElement;
  btnCopyPrompt: HTMLButtonElement;
  btnDownloadResult: HTMLButtonElement;
  btnOpenInWorkspace: HTMLButtonElement;
  btnResetConversion: HTMLButtonElement;
  historyCard: HTMLElement;
  historyToggle: HTMLElement;
  historyCount: HTMLSpanElement;
  historyArrow: HTMLSpanElement;
  historyList: HTMLDivElement;
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
    pageSummaryCard: el<HTMLElement>("pageSummaryCard"),
    archetypeBadge: el<HTMLSpanElement>("archetypeBadge"),
    tabTitle: el<HTMLDivElement>("tabTitle"),
    tabUrl: el<HTMLDivElement>("tabUrl"),
    conversionOptionsCard: el<HTMLElement>("conversionOptionsCard"),
    formatSelect: el<HTMLSelectElement>("formatSelect"),
    budgetChips: el<HTMLDivElement>("budgetChips"),
    actionSection: el<HTMLElement>("actionSection"),
    btnConvert: el<HTMLButtonElement>("btnConvert"),
    btnConvertText: el<HTMLSpanElement>("btnConvertText"),
    spinner: el<HTMLSpanElement>("spinner"),
    statusMessage: el<HTMLDivElement>("statusMessage"),
    progressContainer: el<HTMLDivElement>("progressContainer"),
    progressBar: el<HTMLDivElement>("progressBar"),
    progressPhaseText: el<HTMLDivElement>("progressPhaseText"),
    resultContainer: el<HTMLElement>("resultContainer"),
    resTokens: el<HTMLSpanElement>("resTokens"),
    resChars: el<HTMLSpanElement>("resChars"),
    resFormat: el<HTMLSpanElement>("resFormat"),
    btnCopyResult: el<HTMLButtonElement>("btnCopyResult"),
    btnCopyPrompt: el<HTMLButtonElement>("btnCopyPrompt"),
    btnDownloadResult: el<HTMLButtonElement>("btnDownloadResult"),
    btnOpenInWorkspace: el<HTMLButtonElement>("btnOpenInWorkspace"),
    btnResetConversion: el<HTMLButtonElement>("btnResetConversion"),
    historyCard: el<HTMLElement>("historyCard"),
    historyToggle: el<HTMLElement>("historyToggle"),
    historyCount: el<HTMLSpanElement>("historyCount"),
    historyArrow: el<HTMLSpanElement>("historyArrow"),
    historyList: el<HTMLDivElement>("historyList"),
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
  private lastResult: ConversionResult | null = null;
  private historyOpen = false;

  constructor(elements: PopupElements) {
    this.elements = elements;
  }

  public async init(): Promise<void> {
    this.bindEvents();
    await this.loadInitialState();
    await this.refreshHistory();
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
      navigateToUrl(targetUrl);
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

    // Copy raw content
    this.elements.btnCopyResult.addEventListener("click", async () => {
      if (this.lastResult?.content) {
        const success = await copyToClipboard(this.lastResult.content);
        if (success) {
          const original = this.elements.btnCopyResult.textContent;
          this.elements.btnCopyResult.textContent = "✓ Copied Raw!";
          setTimeout(() => {
            this.elements.btnCopyResult.textContent = original;
          }, 2000);
        }
      }
    });

    // Copy formatted AI prompt
    this.elements.btnCopyPrompt.addEventListener("click", async () => {
      if (this.lastResult?.content) {
        const promptText = formatContentForClipboard(
          this.lastResult.content,
          this.lastResult.url,
          this.currentTab?.title || "Webpage Document",
          this.lastResult.format,
          { mode: "prompt" }
        );
        const success = await copyToClipboard(promptText);
        if (success) {
          const original = this.elements.btnCopyPrompt.textContent;
          this.elements.btnCopyPrompt.textContent = "✨ Prompt Copied!";
          setTimeout(() => {
            this.elements.btnCopyPrompt.textContent = original;
          }, 2000);
        }
      }
    });

    // Download result file
    this.elements.btnDownloadResult.addEventListener("click", async () => {
      if (this.lastResult) {
        const filename = sanitizeDownloadFilename(
          this.currentTab?.title || this.lastResult.url,
          this.lastResult.jobId,
          this.lastResult.format
        );
        await triggerFileDownload(this.lastResult.content, filename, this.lastResult.format);
      }
    });

    // Open in Web UI Workspace
    this.elements.btnOpenInWorkspace.addEventListener("click", () => {
      if (this.lastResult) {
        const workspaceUrl = buildWorkspaceUrl(this.currentConfig.apiUrl, this.lastResult.jobId);
        navigateToUrl(workspaceUrl);
      }
    });

    // Reset conversion view to convert another
    this.elements.btnResetConversion.addEventListener("click", () => {
      this.resetToIdle();
    });

    // Toggle history section
    this.elements.historyToggle.addEventListener("click", () => {
      this.historyOpen = !this.historyOpen;
      this.elements.historyList.style.display = this.historyOpen ? "flex" : "none";
      this.elements.historyArrow.textContent = this.historyOpen ? "▼" : "▶";
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
        const err = sanitizeError((res.success && res.data.error) || (!res.success && res.error) || "Offline");
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

  public updateProgress(progress: { pct: number; phaseText: string }): void {
    this.elements.progressContainer.style.display = "block";
    this.elements.progressBar.style.width = `${progress.pct}%`;
    this.elements.progressPhaseText.textContent = progress.phaseText;
  }

  public showResult(result: ConversionResult): void {
    this.lastResult = result;
    this.elements.progressContainer.style.display = "none";
    this.elements.conversionOptionsCard.style.display = "none";
    this.elements.actionSection.style.display = "none";
    this.elements.resultContainer.style.display = "block";

    this.elements.resTokens.textContent = result.tokenCount.toLocaleString();
    this.elements.resChars.textContent = this.formatBytes(result.charCount);
    this.elements.resFormat.textContent = result.format.toUpperCase();

    this.refreshHistory();
  }

  public resetToIdle(): void {
    this.lastResult = null;
    this.elements.resultContainer.style.display = "none";
    this.elements.progressContainer.style.display = "none";
    this.elements.conversionOptionsCard.style.display = "block";
    this.elements.actionSection.style.display = "block";
    this.elements.btnConvert.disabled = false;
    this.elements.btnConvert.classList.remove("converting");
    this.elements.btnConvertText.textContent = "Convert Current Tab";
    this.elements.statusMessage.className = "status-message";
    this.elements.statusMessage.textContent = "";
  }

  public async refreshHistory(): Promise<void> {
    const res = await sendBackgroundMessage<HistoryItem[]>({ type: "GET_HISTORY" });
    const items = res.success && Array.isArray(res.data) ? res.data : [];

    this.elements.historyCount.textContent = String(items.length);

    if (items.length === 0) {
      this.elements.historyList.innerHTML = `<div class="history-empty">No recent conversions yet</div>`;
      return;
    }

    if (typeof document === "undefined") {
      return;
    }

    this.elements.historyList.innerHTML = "";
    for (const item of items.slice(0, 5)) {
      const itemEl = document.createElement("div");
      itemEl.className = "history-item";

      const detailsEl = document.createElement("div");
      detailsEl.className = "history-details";

      const titleEl = document.createElement("div");
      titleEl.className = "history-title";
      titleEl.textContent = item.title || item.url;
      titleEl.title = item.url;

      const metaEl = document.createElement("div");
      metaEl.className = "history-meta";
      metaEl.textContent = `${item.format.toUpperCase()} • ${item.tokenCount.toLocaleString()} tokens`;

      detailsEl.appendChild(titleEl);
      detailsEl.appendChild(metaEl);

      const actionsEl = document.createElement("div");
      actionsEl.className = "history-actions";

      const btnWorkspace = document.createElement("button");
      btnWorkspace.className = "history-btn";
      btnWorkspace.textContent = "Open";
      btnWorkspace.title = "Open in web workspace";
      btnWorkspace.addEventListener("click", () => {
        const url = buildWorkspaceUrl(this.currentConfig.apiUrl, item.jobId);
        navigateToUrl(url);
      });

      actionsEl.appendChild(btnWorkspace);
      itemEl.appendChild(detailsEl);
      itemEl.appendChild(actionsEl);

      this.elements.historyList.appendChild(itemEl);
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
    this.elements.statusMessage.textContent = "";

    this.updateProgress({ pct: 15, phaseText: "Initiating conversion..." });

    try {
      const format = this.elements.formatSelect.value as OutputFormat;
      const res = await sendBackgroundMessage<ConversionResult>({
        type: "START_CONVERSION",
        url: this.currentTab.url,
        format,
        tokenBudget: this.selectedBudget,
        model: this.currentConfig.defaultTargetModel,
        sendDomSnapshot: this.currentConfig.sendDomSnapshot,
        tabId: this.currentTab.id,
      });

      if (res.success && res.data) {
        this.updateProgress({ pct: 100, phaseText: "Conversion complete!" });
        this.showResult(res.data);
      } else {
        const errorMsg = sanitizeError(!res.success ? res.error : "Conversion failed without error output");
        this.elements.progressContainer.style.display = "none";
        this.elements.statusMessage.className = "status-message error";
        this.elements.statusMessage.textContent = `Conversion failed: ${errorMsg}`;
        this.elements.btnConvertText.textContent = "Retry Conversion";
        this.elements.btnConvert.disabled = false;
      }
    } catch (err: any) {
      this.elements.progressContainer.style.display = "none";
      this.elements.statusMessage.className = "status-message error";
      this.elements.statusMessage.textContent = sanitizeError(err.message || "Failed to initiate conversion");
      this.elements.btnConvertText.textContent = "Retry Conversion";
      this.elements.btnConvert.disabled = false;
    } finally {
      this.isConverting = false;
      this.elements.btnConvert.classList.remove("converting");
    }
  }

  private formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
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
