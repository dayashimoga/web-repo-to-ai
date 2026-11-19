import { beforeEach, describe, expect, it } from "vitest";
import {
  addHistoryItem,
  clearHistory,
  HISTORY_STORAGE_KEY,
  loadHistory,
  MAX_HISTORY_ITEMS,
  removeHistoryItem,
} from "../src/storage/history";
import { setupMockChrome } from "./mocks/chrome";

describe("History Storage (apps/extension)", () => {
  let mockChrome: ReturnType<typeof setupMockChrome>;

  beforeEach(() => {
    mockChrome = setupMockChrome();
  });

  it("returns empty array when history storage is empty", async () => {
    const history = await loadHistory();
    expect(history).toEqual([]);
  });

  it("adds an item to history and assigns id and timestamp", async () => {
    const updated = await addHistoryItem({
      jobId: "job-001",
      url: "https://github.com/rust-lang/rust",
      title: "rust-lang/rust",
      format: "markdown",
      tokenCount: 4200,
      charCount: 16800,
    });

    expect(updated).toHaveLength(1);
    expect(updated[0].jobId).toBe("job-001");
    expect(updated[0].id).toBeDefined();
    expect(updated[0].timestamp).toBeDefined();

    // Verify stored
    const loaded = await loadHistory();
    expect(loaded).toHaveLength(1);
    expect(loaded[0].url).toBe("https://github.com/rust-lang/rust");
  });

  it("deduplicates items with identical jobId", async () => {
    await addHistoryItem({
      jobId: "job-dup",
      url: "https://example.com/page",
      title: "Initial Title",
      format: "markdown",
      tokenCount: 100,
      charCount: 400,
    });

    const updated = await addHistoryItem({
      jobId: "job-dup",
      url: "https://example.com/page",
      title: "Updated Title",
      format: "markdown",
      tokenCount: 120,
      charCount: 480,
    });

    expect(updated).toHaveLength(1);
    expect(updated[0].title).toBe("Updated Title");
  });

  it("caps history at MAX_HISTORY_ITEMS", async () => {
    for (let i = 1; i <= MAX_HISTORY_ITEMS + 5; i++) {
      await addHistoryItem({
        jobId: `job-bulk-${i}`,
        url: `https://example.com/page/${i}`,
        title: `Page ${i}`,
        format: "markdown",
        tokenCount: 100 * i,
        charCount: 400 * i,
      });
    }

    const loaded = await loadHistory();
    expect(loaded).toHaveLength(MAX_HISTORY_ITEMS);
    // Newest should be at front
    expect(loaded[0].jobId).toBe(`job-bulk-${MAX_HISTORY_ITEMS + 5}`);
  });

  it("removes item by ID", async () => {
    const items = await addHistoryItem({
      id: "hist-remove-me",
      jobId: "job-rem",
      url: "https://example.com/rem",
      title: "Remove Me",
      format: "markdown",
      tokenCount: 50,
      charCount: 200,
    });
    expect(items).toHaveLength(1);

    const remaining = await removeHistoryItem("hist-remove-me");
    expect(remaining).toHaveLength(0);

    const loaded = await loadHistory();
    expect(loaded).toHaveLength(0);
  });

  it("clears entire history", async () => {
    await addHistoryItem({
      jobId: "job-c1",
      url: "https://example.com/1",
      title: "One",
      format: "markdown",
      tokenCount: 10,
      charCount: 40,
    });
    await addHistoryItem({
      jobId: "job-c2",
      url: "https://example.com/2",
      title: "Two",
      format: "markdown",
      tokenCount: 20,
      charCount: 80,
    });

    expect(await loadHistory()).toHaveLength(2);

    await clearHistory();
    expect(await loadHistory()).toHaveLength(0);
  });
});
