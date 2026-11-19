// High-fidelity Mock of Chrome WebExtensions Manifest V3 APIs for Testing

export interface MockStorageArea {
  data: Record<string, unknown>;
  get: (keys: any, callback?: (items: any) => void) => Promise<any>;
  set: (items: Record<string, unknown>, callback?: () => void) => Promise<void>;
  remove: (keys: any, callback?: () => void) => Promise<void>;
  clear: (callback?: () => void) => Promise<void>;
}

export function createMockStorageArea(): MockStorageArea {
  const store: Record<string, unknown> = {};

  return {
    data: store,
    get: async (keys: any, callback?: (items: any) => void) => {
      let result: Record<string, unknown> = {};
      if (typeof keys === "string") {
        if (store[keys] !== undefined) {
          result[keys] = store[keys];
        }
      } else if (Array.isArray(keys)) {
        for (const k of keys) {
          if (store[k] !== undefined) {
            result[k] = store[k];
          }
        }
      } else if (keys && typeof keys === "object") {
        for (const k of Object.keys(keys)) {
          result[k] = store[k] !== undefined ? store[k] : keys[k];
        }
      } else {
        result = { ...store };
      }

      if (callback) {
        callback(result);
      }
      return result;
    },
    set: async (items: Record<string, unknown>, callback?: () => void) => {
      Object.assign(store, items);
      if (callback) {
        callback();
      }
    },
    remove: async (keys: any, callback?: () => void) => {
      const arr = Array.isArray(keys) ? keys : [keys];
      for (const k of arr) {
        delete store[k];
      }
      if (callback) {
        callback();
      }
    },
    clear: async (callback?: () => void) => {
      for (const k of Object.keys(store)) {
        delete store[k];
      }
      if (callback) {
        callback();
      }
    },
  };
}

export interface MockChrome {
  storage: {
    sync: MockStorageArea;
    local: MockStorageArea;
  };
  runtime: {
    onInstalled: {
      listeners: ((details: any) => void)[];
      addListener: (cb: (details: any) => void) => void;
      trigger: (details: any) => void;
    };
    onMessage: {
      listeners: ((msg: any, sender: any, sendResponse: any) => boolean | void)[];
      addListener: (cb: (msg: any, sender: any, sendResponse: any) => boolean | void) => void;
      trigger: (msg: any, sender?: any) => Promise<any>;
    };
    sendMessage: (msg: any, callback?: (response: any) => void) => Promise<any>;
    openOptionsPage: () => Promise<void>;
    lastError: { message?: string } | undefined;
  };
  tabs: {
    mockTabs: any[];
    query: (queryInfo: any) => Promise<any[]>;
    create: (props: any) => Promise<any>;
    sendMessage: (tabId: number, msg: any) => Promise<any>;
  };
  action: {
    badgeText: string;
    badgeBackgroundColor: string;
    setBadgeText: (details: { text: string }) => Promise<void>;
    setBadgeBackgroundColor: (details: { color: string }) => Promise<void>;
  };
}

export function createMockChrome(): MockChrome {
  const syncStorage = createMockStorageArea();
  const localStorageArea = createMockStorageArea();

  const mockInstalledListeners: ((details: any) => void)[] = [];
  const mockMessageListeners: ((msg: any, sender: any, sendResponse: any) => boolean | void)[] = [];

  const mock: MockChrome = {
    storage: {
      sync: syncStorage,
      local: localStorageArea,
    },
    runtime: {
      onInstalled: {
        listeners: mockInstalledListeners,
        addListener: (cb) => mockInstalledListeners.push(cb),
        trigger: (details) => mockInstalledListeners.forEach((cb) => cb(details)),
      },
      onMessage: {
        listeners: mockMessageListeners,
        addListener: (cb) => mockMessageListeners.push(cb),
        trigger: async (msg, sender = {}) => {
          if (mockMessageListeners.length === 0) {
            const { handleMessage } = await import("../../src/background/index");
            return await handleMessage(msg);
          }
          return new Promise((resolve) => {
            const sendResponse = (res: any) => {
              resolve(res);
            };
            for (const listener of mockMessageListeners) {
              listener(msg, sender, sendResponse);
            }
          });
        },
      },
      sendMessage: async (msg: any, callback?: (response: any) => void) => {
        const response = await mock.runtime.onMessage.trigger(msg);
        if (callback) {
          callback(response);
        }
        return response;
      },
      openOptionsPage: async () => {},
      lastError: undefined,
    },
    tabs: {
      mockTabs: [
        {
          id: 101,
          index: 0,
          windowId: 1,
          highlighted: true,
          active: true,
          pinned: false,
          url: "https://github.com/rust-lang/rust",
          title: "rust-lang/rust: Empowering everyone to build reliable and efficient software.",
          favIconUrl: "https://github.com/favicon.ico",
          status: "complete",
        },
      ],
      query: async (_queryInfo: any) => {
        return mock.tabs.mockTabs.filter((t) => t.active);
      },
      create: async (props: any) => {
        const newTab = {
          id: Math.floor(Math.random() * 1000) + 1,
          index: mock.tabs.mockTabs.length,
          windowId: 1,
          highlighted: true,
          active: true,
          pinned: false,
          url: props.url || "",
          title: "New Tab",
          status: "complete",
        };
        mock.tabs.mockTabs.push(newTab);
        return newTab;
      },
      sendMessage: async (_tabId: number, _msg: any) => ({ success: true }),
    },
    action: {
      badgeText: "",
      badgeBackgroundColor: "",
      setBadgeText: async (details: { text: string }) => {
        mock.action.badgeText = details.text;
      },
      setBadgeBackgroundColor: async (details: { color: string }) => {
        mock.action.badgeBackgroundColor = details.color;
      },
    },
  };

  return mock;
}

/**
 * Installs mockChrome onto the global `chrome` object.
 */
export function setupMockChrome(): MockChrome {
  const mock = createMockChrome();
  (globalThis as any).chrome = mock;
  return mock;
}
