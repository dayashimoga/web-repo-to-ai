// Conversion History Manager for web-repo-to-ai Browser Extension

import { HistoryItem } from "../types/index";

export const HISTORY_STORAGE_KEY = "web_repo_to_ai_history";
export const MAX_HISTORY_ITEMS = 20;

function getStorageArea(): {
  get: (key: string) => Promise<Record<string, unknown>>;
  set: (items: Record<string, unknown>) => Promise<void>;
  remove: (key: string) => Promise<void>;
} {
  if (
    typeof chrome !== "undefined" &&
    chrome.storage &&
    chrome.storage.local &&
    typeof chrome.storage.local.get === "function"
  ) {
    return {
      get: (key: string) =>
        new Promise((resolve) => {
          chrome.storage.local.get(key, (items) => resolve(items || {}));
        }),
      set: (items: Record<string, unknown>) =>
        new Promise((resolve) => {
          chrome.storage.local.set(items, () => resolve());
        }),
      remove: (key: string) =>
        new Promise((resolve) => {
          chrome.storage.local.remove(key, () => resolve());
        }),
    };
  }

  // In-memory fallback
  const memoryStore: Record<string, unknown> = {};
  return {
    get: async (key: string) => {
      const val = memoryStore[key];
      return val ? { [key]: val } : {};
    },
    set: async (items: Record<string, unknown>) => {
      Object.assign(memoryStore, items);
    },
    remove: async (key: string) => {
      delete memoryStore[key];
    },
  };
}

/**
 * Loads recent conversion history, sorted newest first.
 */
export async function loadHistory(): Promise<HistoryItem[]> {
  try {
    const storage = getStorageArea();
    const result = await storage.get(HISTORY_STORAGE_KEY);
    const items = (result[HISTORY_STORAGE_KEY] || []) as HistoryItem[];
    return Array.isArray(items) ? items : [];
  } catch (err) {
    console.warn("Failed to load conversion history", err);
    return [];
  }
}

/**
 * Adds an item to conversion history, deduplicating and maintaining max ceiling.
 */
export async function addHistoryItem(
  item: Omit<HistoryItem, "id" | "timestamp"> & {
    id?: string;
    timestamp?: string;
  }
): Promise<HistoryItem[]> {
  const current = await loadHistory();
  const id = item.id || `hist_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
  const timestamp = item.timestamp || new Date().toISOString();

  const newItem: HistoryItem = {
    ...item,
    id,
    timestamp,
  };

  // Filter out any existing item with identical jobId or URL
  const filtered = current.filter((h) => h.jobId !== item.jobId);
  const updated = [newItem, ...filtered].slice(0, MAX_HISTORY_ITEMS);

  const storage = getStorageArea();
  await storage.set({ [HISTORY_STORAGE_KEY]: updated });
  return updated;
}

/**
 * Removes a specific history item by ID.
 */
export async function removeHistoryItem(id: string): Promise<HistoryItem[]> {
  const current = await loadHistory();
  const updated = current.filter((h) => h.id !== id);
  const storage = getStorageArea();
  await storage.set({ [HISTORY_STORAGE_KEY]: updated });
  return updated;
}

/**
 * Clears all saved conversion history.
 */
export async function clearHistory(): Promise<void> {
  const storage = getStorageArea();
  await storage.remove(HISTORY_STORAGE_KEY);
}
