// Chrome Extension API Type Declarations (Manifest V3)

declare namespace chrome {
  namespace storage {
    interface StorageArea {
      get(
        keys?: string | string[] | Record<string, unknown> | null
      ): Promise<Record<string, unknown>>;
      get(callback: (items: Record<string, unknown>) => void): void;
      get(
        keys: string | string[] | Record<string, unknown> | null,
        callback: (items: Record<string, unknown>) => void
      ): void;

      set(items: Record<string, unknown>): Promise<void>;
      set(items: Record<string, unknown>, callback: () => void): void;

      remove(keys: string | string[]): Promise<void>;
      remove(keys: string | string[], callback: () => void): void;

      clear(): Promise<void>;
      clear(callback: () => void): void;
    }

    const sync: StorageArea;
    const local: StorageArea;
  }

  namespace runtime {
    interface MessageSender {
      id?: string;
      tab?: tabs.Tab;
      url?: string;
      frameId?: number;
    }

    interface InstalledDetails {
      reason: "install" | "update" | "chrome_update" | "shared_module_update";
      previousVersion?: string;
    }

    interface RuntimeEvent<T extends (...args: any[]) => any> {
      addListener(callback: T): void;
      removeListener(callback: T): void;
      hasListener(callback: T): boolean;
    }

    const onInstalled: RuntimeEvent<(details: InstalledDetails) => void>;
    const onMessage: RuntimeEvent<
      (
        message: any,
        sender: MessageSender,
        sendResponse: (response?: any) => void
      ) => boolean | void | Promise<any>
    >;

    function sendMessage<M = any, R = any>(message: M): Promise<R>;
    function sendMessage<M = any, R = any>(
      message: M,
      responseCallback: (response: R) => void
    ): void;
    function openOptionsPage(): Promise<void>;
    function getURL(path: string): string;

    let lastError: { message?: string } | undefined;
  }

  namespace tabs {
    interface Tab {
      id?: number;
      index: number;
      windowId: number;
      highlighted: boolean;
      active: boolean;
      pinned: boolean;
      url?: string;
      title?: string;
      favIconUrl?: string;
      status?: "loading" | "complete";
      incognito?: boolean;
      width?: number;
      height?: number;
    }

    interface QueryInfo {
      active?: boolean;
      currentWindow?: boolean;
      lastFocusedWindow?: boolean;
      windowId?: number;
      url?: string | string[];
      title?: string;
    }

    function query(queryInfo: QueryInfo): Promise<Tab[]>;
    function create(createProperties: { url?: string; active?: boolean }): Promise<Tab>;
    function sendMessage<M = any, R = any>(tabId: number, message: M): Promise<R>;
  }

  namespace action {
    function setBadgeText(details: { text: string; tabId?: number }): Promise<void>;
    function setBadgeBackgroundColor(details: {
      color: string | [number, number, number, number];
      tabId?: number;
    }): Promise<void>;
    function setTitle(details: { title: string; tabId?: number }): Promise<void>;
  }

  namespace scripting {
    interface ScriptInjection<Args extends any[] = any[], Result = any> {
      target: { tabId: number; allFrames?: boolean; frameIds?: number[] };
      func?: (...args: Args) => Result;
      args?: Args;
      files?: string[];
    }

    interface InjectionResult<Result = any> {
      frameId: number;
      result: Result;
    }

    function executeScript<Args extends any[] = any[], Result = any>(
      injection: ScriptInjection<Args, Result>
    ): Promise<InjectionResult<Result>[]>;
  }

  namespace downloads {
    interface DownloadOptions {
      url: string;
      filename?: string;
      conflictAction?: "uniquify" | "overwrite" | "prompt";
      saveAs?: boolean;
    }

    function download(options: DownloadOptions): Promise<number>;
  }
}
