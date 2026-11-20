// Background Service Worker for web-repo-to-ai Browser Extension (Manifest V3)

import { ExtensionApiClient } from "../api/client";
import { ConversionStateMachine } from "../conversion/state";
import { captureActiveTabDOM } from "../dom/extractor";
import { copyToClipboard } from "../handoff/clipboard";
import { navigateToUrl } from "../handoff/deeplink";
import { isPrivilegedUrl, validateTargetUrl } from "../security/policy";
import { sanitizeError, sanitizeLog } from "../security/redaction";
import { loadConfig, resetConfig, saveConfig } from "../storage/config";
import { addHistoryItem, clearHistory, loadHistory } from "../storage/history";
import {
  BackendStatus,
  DetectedArchetype,
  ExtensionConfig,
  ExtensionMessage,
  ExtensionResponse,
  HistoryItem,
  SourceClassification,
  TabInfo,
} from "../types/index";

export const globalConversionStateMachine = new ConversionStateMachine();

/**
 * Classifies a URL into a source archetype using heuristics.
 */
export function classifyUrl(urlString: string): SourceClassification {
  if (!urlString || typeof urlString !== "string") {
    return {
      archetype: "unknown",
      displayName: "Unknown Source",
      url: "",
      isSupported: false,
    };
  }

  let parsed: URL;
  try {
    parsed = new URL(urlString);
  } catch {
    return {
      archetype: "unknown",
      displayName: "Invalid URL",
      url: urlString,
      isSupported: false,
    };
  }

  // Unsupported or privileged browser internal schemes
  if (isPrivilegedUrl(urlString)) {
    return {
      archetype: "unknown",
      displayName: "Internal Browser Page",
      url: urlString,
      isSupported: false,
      details: { scheme: parsed.protocol },
    };
  }

  const hostname = parsed.hostname.toLowerCase();
  const pathname = parsed.pathname;

  // GitHub repository detection
  if (hostname === "github.com") {
    const parts = pathname.split("/").filter(Boolean);
    const nonRepoRoutes = new Set([
      "settings",
      "notifications",
      "explore",
      "trending",
      "about",
      "pricing",
      "features",
      "enterprise",
      "security",
      "login",
      "signup",
    ]);

    if (parts.length >= 2 && !nonRepoRoutes.has(parts[0].toLowerCase())) {
      const owner = parts[0];
      const repo = parts[1];
      return {
        archetype: "github_repository",
        displayName: `GitHub: ${owner}/${repo}`,
        url: urlString,
        isSupported: true,
        details: { owner, repo, subpath: parts.slice(2).join("/") },
      };
    }
  }

  // File resources
  const lowerPath = pathname.toLowerCase();
  const fileExts = [".json", ".yaml", ".yml", ".toml", ".xml", ".txt", ".csv", ".md"];
  for (const ext of fileExts) {
    if (lowerPath.endsWith(ext)) {
      return {
        archetype: "file_resource",
        displayName: `Structured File (${ext.toUpperCase().replace(".", "")})`,
        url: urlString,
        isSupported: true,
      };
    }
  }

  // Documentation detection
  if (
    hostname.startsWith("docs.") ||
    hostname.startsWith("documentation.") ||
    hostname.includes("readthedocs.io") ||
    hostname.includes("gitbook.io") ||
    lowerPath.includes("/docs/") ||
    lowerPath.includes("/documentation/") ||
    lowerPath.includes("/guide/") ||
    lowerPath.includes("/manual/")
  ) {
    return {
      archetype: "web_documentation",
      displayName: "Technical Documentation",
      url: urlString,
      isSupported: true,
    };
  }

  // Web article / blog detection
  if (
    hostname.includes("medium.com") ||
    hostname.includes("dev.to") ||
    hostname.includes("substack.com") ||
    lowerPath.includes("/blog/") ||
    lowerPath.includes("/article/") ||
    lowerPath.includes("/posts/") ||
    lowerPath.includes("/news/")
  ) {
    return {
      archetype: "web_article",
      displayName: "Web Article / Blog",
      url: urlString,
      isSupported: true,
    };
  }

  // Generic web page
  return {
    archetype: "generic_web",
    displayName: "Webpage",
    url: urlString,
    isSupported: true,
  };
}

/**
 * Checks connectivity and health status of the backend API instance.
 */
export async function checkBackendStatus(apiUrl?: string, apiKey?: string): Promise<BackendStatus> {
  const targetUrl = apiUrl || (await loadConfig()).apiUrl;
  const targetKey = apiKey !== undefined ? apiKey : (await loadConfig()).apiKey;

  const normalized = targetUrl.replace(/\/+$/, "");
  const startTime = Date.now();

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 5000);

  const headers: Record<string, string> = {
    Accept: "application/json",
  };
  if (targetKey) {
    headers["Authorization"] = `Bearer ${targetKey}`;
  }

  try {
    // Attempt standard health endpoint: /api/v1/health, falling back to /health
    let res: Response;
    try {
      res = await fetch(`${normalized}/api/v1/health`, {
        method: "GET",
        headers,
        signal: controller.signal,
      });
      if (res.status === 404) {
        res = await fetch(`${normalized}/health`, {
          method: "GET",
          headers,
          signal: controller.signal,
        });
      }
    } catch (err: any) {
      if (controller.signal.aborted) {
        throw new Error("Connection timed out after 5000ms");
      }
      throw err;
    } finally {
      clearTimeout(timeoutId);
    }

    const latencyMs = Date.now() - startTime;

    if (!res.ok) {
      await updateActionBadge(false);
      return {
        connected: false,
        latencyMs,
        error: `Server returned HTTP ${res.status}: ${res.statusText}`,
      };
    }

    let payload: any = {};
    try {
      payload = await res.json();
    } catch {
      payload = { status: "ok" };
    }

    await updateActionBadge(true);
    return {
      connected: true,
      version: payload.version || "unknown",
      status: payload.status || "healthy",
      latencyMs,
    };
  } catch (err: any) {
    const latencyMs = Date.now() - startTime;
    await updateActionBadge(false);
    return {
      connected: false,
      latencyMs,
      error: sanitizeError(err.message || "Failed to reach web-repo-to-ai backend"),
    };
  }
}

/**
 * Updates the extension badge indicator in the browser toolbar.
 */
export async function updateActionBadge(connected: boolean): Promise<void> {
  if (typeof chrome === "undefined" || !chrome.action) {
    return;
  }
  try {
    if (connected) {
      await chrome.action.setBadgeText({ text: "" });
    } else {
      await chrome.action.setBadgeText({ text: "ERR" });
      await chrome.action.setBadgeBackgroundColor({ color: "#ef4444" });
    }
  } catch {
    // Ignore badge errors in unsupported or headless environments
  }
}

/**
 * Queries the active tab in the currently focused window.
 */
export async function getActiveTab(): Promise<TabInfo | null> {
  if (typeof chrome === "undefined" || !chrome.tabs || !chrome.tabs.query) {
    return null;
  }

  const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tabs || tabs.length === 0) {
    return null;
  }

  const active = tabs[0];
  return {
    id: active.id,
    url: active.url || "",
    title: active.title || "",
    favIconUrl: active.favIconUrl,
  };
}

/**
 * Handles incoming messages from popup, options, or content scripts.
 */
export async function handleMessage(message: ExtensionMessage): Promise<ExtensionResponse> {
  if (!message || typeof message !== "object" || !("type" in message)) {
    return { success: false, error: "Invalid message payload: missing 'type'" };
  }

  try {
    switch (message.type) {
      case "PING":
        return {
          success: true,
          data: {
            status: "pong",
            timestamp: Date.now(),
          },
        };

      case "CHECK_BACKEND_STATUS": {
        const status = await checkBackendStatus(message.apiUrl, message.apiKey);
        return { success: true, data: status };
      }

      case "GET_CONFIG": {
        const config = await loadConfig();
        return { success: true, data: config };
      }

      case "SAVE_CONFIG": {
        const saved = await saveConfig(message.config);
        return { success: true, data: saved };
      }

      case "RESET_CONFIG": {
        const reset = await resetConfig();
        return { success: true, data: reset };
      }

      case "GET_ACTIVE_TAB": {
        const tab = await getActiveTab();
        if (!tab) {
          return { success: false, error: "No active tab found" };
        }
        return { success: true, data: tab };
      }

      case "DETECT_URL_TYPE": {
        const classification = classifyUrl(message.url);
        return { success: true, data: classification };
      }

      case "CAPTURE_DOM": {
        const tab = await getActiveTab();
        const tabId = message.tabId !== undefined ? message.tabId : tab?.id;
        const snapshot = await captureActiveTabDOM(tabId, tab?.url);
        return { success: true, data: snapshot };
      }

      case "START_CONVERSION": {
        const classification = classifyUrl(message.url);
        if (!classification.isSupported || isPrivilegedUrl(message.url)) {
          return {
            success: false,
            error: `URL is not supported for conversion: ${sanitizeError(message.url)}`,
          };
        }
        const config = await loadConfig();
        const client = new ExtensionApiClient({
          baseUrl: config.apiUrl,
          apiKey: config.apiKey,
        });

        const result = await globalConversionStateMachine.executeConversion({
          url: message.url,
          format: message.format,
          tokenBudget: message.tokenBudget,
          model: message.model,
          sendDomSnapshot: message.sendDomSnapshot ?? config.sendDomSnapshot,
          tabId: message.tabId,
          apiClient: client,
        });

        // Automatically record to history
        try {
          const tab = await getActiveTab();
          await addHistoryItem({
            jobId: result.jobId,
            url: result.url,
            title: tab?.title || result.url,
            format: result.format,
            tokenCount: result.tokenCount,
            charCount: result.charCount,
            downloadUrl: result.downloadUrl,
          });
        } catch (histErr) {
          console.warn("Failed to auto-record conversion history item:", histErr);
        }

        return {
          success: true,
          data: result,
        };
      }

      case "GET_CONVERSION_STATUS": {
        return {
          success: true,
          data: globalConversionStateMachine.getState(),
        };
      }

      case "GET_HISTORY": {
        const history = await loadHistory();
        return { success: true, data: history };
      }

      case "SAVE_HISTORY_ITEM": {
        const history = await addHistoryItem(message.item);
        return { success: true, data: history };
      }

      case "CLEAR_HISTORY": {
        await clearHistory();
        return { success: true, data: [] };
      }

      case "COPY_TO_CLIPBOARD": {
        const copied = await copyToClipboard(message.text);
        return { success: true, data: { copied } };
      }

      case "DOWNLOAD_ARTIFACT": {
        if (
          typeof chrome !== "undefined" &&
          chrome.downloads &&
          typeof chrome.downloads.download === "function"
        ) {
          const downloadId = await chrome.downloads.download({
            url: message.url,
            filename: message.filename,
            saveAs: false,
          });
          return { success: true, data: { downloadId } };
        }
        return { success: false, error: "chrome.downloads API not available" };
      }

      case "NAVIGATE_WORKSPACE": {
        await navigateToUrl(message.url);
        return { success: true, data: { navigated: true } };
      }

      default: {
        const unhandled = message as any;
        return {
          success: false,
          error: `Unhandled message type: ${unhandled.type}`,
        };
      }
    }
  } catch (err: any) {
    return {
      success: false,
      error: sanitizeError(
        err.message || "An unexpected error occurred in background service worker"
      ),
    };
  }
}

// Register Chrome Extension Lifecycle Listeners
if (typeof chrome !== "undefined" && chrome.runtime) {
  if (chrome.runtime.onInstalled) {
    chrome.runtime.onInstalled.addListener(async (details) => {
      console.log("[web-repo-to-ai] Background worker initialized:", details.reason);
      try {
        await loadConfig();
        await checkBackendStatus();
      } catch (err) {
        console.warn("[web-repo-to-ai] Initial background setup warning:", sanitizeError(err));
      }
    });
  }

  if (chrome.runtime.onMessage) {
    chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
      handleMessage(message as ExtensionMessage)
        .then((res) => sendResponse(res))
        .catch((err) =>
          sendResponse({
            success: false,
            error: sanitizeError(err.message || "Internal service worker failure"),
          })
        );
      return true; // Keep message channel open for async response
    });
  }
}
