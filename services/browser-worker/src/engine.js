/**
 * @file engine.js
 * @description Playwright / Chromium dynamic browser rendering engine with lifecycle
 * management, concurrency bounds, hard session deadlines, per-request context isolation,
 * dialog auto-dismissal, and hardened SSRF subresource interception.
 */

import { chromium } from 'playwright-core';
import { createRenderResponse } from './protocol.js';
import { ConcurrencyLimiter } from './limiter.js';

export const DEFAULT_CHROMIUM_ARGS = [
  '--no-sandbox',
  '--disable-setuid-sandbox',
  '--disable-dev-shm-usage',
  '--disable-gpu',
  '--no-first-run',
  '--no-zygote',
  '--disable-extensions',
  '--disable-background-networking',
  '--disable-default-apps',
  '--disable-sync',
  '--disable-translate',
  '--metrics-recording-only',
  '--mute-audio',
  '--safebrowsing-disable-auto-update',
];

export const MAX_CONSOLE_MESSAGES = 100;
export const MAX_PAGE_ERRORS = 50;
export const DEFAULT_MAX_SUBRESOURCES = 200;
export const DEFAULT_MAX_HTML_BYTES = 10 * 1024 * 1024; // 10 MB payload limit
export const DEFAULT_MAX_RENDERS_PER_BROWSER = 200;
export const DEFAULT_MAX_MEMORY_RSS_BYTES = 768 * 1024 * 1024; // 768 MB RSS ceiling
export const DEFAULT_HARD_TIMEOUT_BUFFER_MS = 5000; // 5s buffer over client timeout

/**
 * Custom error class representing rendering execution, security, and timeout failures.
 */
export class RenderEngineError extends Error {
  /**
   * @param {string} code - Machine-readable error code.
   * @param {string} detail - Human-readable explanation.
   * @param {number} [statusCode=500] - Corresponding HTTP status code.
   */
  constructor(code, detail, statusCode = 500) {
    super(detail);
    this.name = 'RenderEngineError';
    this.code = code;
    this.detail = detail;
    this.statusCode = statusCode;
  }
}

/**
 * Manages Chromium browser lifecycle, concurrency queues, resource limits, and isolated page rendering.
 */
export class PlaywrightEngine {
  /**
   * @param {object} [options={}]
   * @param {typeof chromium} [options.chromiumModule] - Dependency injection for testing.
   * @param {string[]} [options.args] - Custom Chromium CLI arguments.
   * @param {string} [options.executablePath] - Custom browser executable path.
   * @param {number} [options.maxConcurrent] - Maximum simultaneous active renders.
   * @param {number} [options.maxQueueSize] - Maximum queue capacity.
   * @param {number} [options.queueTimeoutMs] - Maximum queue wait timeout in ms.
   * @param {number} [options.maxSubresources] - Maximum subresource requests allowed per page.
   * @param {number} [options.maxHtmlBytes] - Maximum rendered HTML payload size in bytes.
   * @param {number} [options.maxRendersPerBrowser] - Lifetime render count before browser recycling.
   * @param {number} [options.maxMemoryRssBytes] - Process RSS memory threshold triggering browser recycling.
   * @param {number} [options.hardTimeoutBufferMs] - Buffer added to request timeout for hard deadline timer.
   */
  constructor(options = {}) {
    this.chromiumModule = options.chromiumModule || chromium;
    this.args = options.args || DEFAULT_CHROMIUM_ARGS;
    this.executablePath = options.executablePath || process.env.CHROME_BIN || process.env.BROWSER_PATH || null;
    this.limiter = new ConcurrencyLimiter({
      maxConcurrent: options.maxConcurrent,
      maxQueueSize: options.maxQueueSize,
      queueTimeoutMs: options.queueTimeoutMs,
    });
    this.maxSubresources =
      typeof options.maxSubresources === 'number'
        ? options.maxSubresources
        : Number.parseInt(process.env.MAX_SUBRESOURCES_PER_PAGE || '', 10) || DEFAULT_MAX_SUBRESOURCES;
    this.maxHtmlBytes = typeof options.maxHtmlBytes === 'number' ? options.maxHtmlBytes : DEFAULT_MAX_HTML_BYTES;
    this.maxRendersPerBrowser =
      typeof options.maxRendersPerBrowser === 'number'
        ? options.maxRendersPerBrowser
        : Number.parseInt(process.env.MAX_RENDERS_PER_BROWSER || '', 10) || DEFAULT_MAX_RENDERS_PER_BROWSER;
    this.maxMemoryRssBytes =
      typeof options.maxMemoryRssBytes === 'number' ? options.maxMemoryRssBytes : DEFAULT_MAX_MEMORY_RSS_BYTES;
    this.hardTimeoutBufferMs =
      typeof options.hardTimeoutBufferMs === 'number' ? options.hardTimeoutBufferMs : DEFAULT_HARD_TIMEOUT_BUFFER_MS;

    this.browser = null;
    this.launchPromise = null;
    this.closed = false;
    this.rendersCount = 0;
    this.totalRendersServed = 0;
  }

  /**
   * Obtains the running Chromium browser instance, launching it lazily if not active.
   *
   * @returns {Promise<import('playwright-core').Browser>}
   */
  async getBrowser() {
    if (this.closed) {
      throw new RenderEngineError('EngineClosed', 'Browser engine has been shut down.', 503);
    }

    if (this.browser && this.browser.isConnected()) {
      return this.browser;
    }

    if (this.launchPromise) {
      return this.launchPromise;
    }

    this.launchPromise = this.launchBrowser();
    try {
      this.browser = await this.launchPromise;
      return this.browser;
    } finally {
      this.launchPromise = null;
    }
  }

  /**
   * Launches Chromium with fallback resolution for containers and local host systems.
   *
   * @private
   * @returns {Promise<import('playwright-core').Browser>}
   */
  async launchBrowser() {
    const baseOptions = {
      headless: true,
      args: this.args,
    };

    if (this.executablePath) {
      baseOptions.executablePath = this.executablePath;
    }

    // 1. Try default launch (uses PLAYWRIGHT_BROWSERS_PATH or configured executable)
    try {
      return await this.chromiumModule.launch(baseOptions);
    } catch (defaultErr) {
      // If executable not found on developer hosts, attempt channel fallbacks
      const isMissingExecutable =
        defaultErr.message.includes("Executable doesn't exist") ||
        defaultErr.message.includes('browserType.launch: Executable');

      if (isMissingExecutable && !this.executablePath) {
        // Try Google Chrome channel
        try {
          return await this.chromiumModule.launch({
            ...baseOptions,
            channel: 'chrome',
          });
        } catch {
          // Try Microsoft Edge channel
          try {
            return await this.chromiumModule.launch({
              ...baseOptions,
              channel: 'msedge',
            });
          } catch {
            throw new RenderEngineError(
              'BrowserLaunchFailed',
              `Could not launch Chromium: ${defaultErr.message}`,
              503
            );
          }
        }
      }

      throw new RenderEngineError(
        'BrowserLaunchFailed',
        `Failed to launch browser: ${defaultErr.message}`,
        503
      );
    }
  }

  /**
   * Recycles the Chromium browser process to free memory and clean up zombie workers.
   *
   * @returns {Promise<void>}
   */
  async recycleBrowser() {
    if (this.browser) {
      try {
        await this.browser.close();
      } catch {
        // Ignored during recycling
      }
      this.browser = null;
      this.rendersCount = 0;
    }
  }

  /**
   * Probes whether the browser is currently operational and responsive.
   *
   * @returns {Promise<boolean>}
   */
  async isReady() {
    if (this.closed) {
      return false;
    }
    try {
      const browser = await this.getBrowser();
      return Boolean(browser && browser.isConnected());
    } catch {
      return false;
    }
  }

  /**
   * Renders a webpage with concurrency bounding, hard session timeouts, and sandbox isolation.
   *
   * @param {import('./protocol.js').RenderRequest} reqData - Validated render request parameters.
   * @returns {Promise<import('./protocol.js').RenderResponse>}
   */
  async render(reqData) {
    // 1. Acquire concurrency token (or wait in bounded queue; throws 429 if full, 504 if queue timeout)
    await this.limiter.acquire();

    let context = null;
    let deadlineTimer = null;

    try {
      const browser = await this.getBrowser();

      // 2. Context isolation & sandbox hardening:
      // - block ServiceWorkers
      // - disable file downloads
      // - revoke all device/sensor permissions
      context = await browser.newContext({
        viewport: reqData.viewport,
        userAgent: reqData.user_agent || undefined,
        extraHTTPHeaders: reqData.headers && Object.keys(reqData.headers).length > 0 ? reqData.headers : undefined,
        javaScriptEnabled: reqData.javascript_enabled !== false,
        ignoreHTTPSErrors: false,
        serviceWorkers: 'block',
        acceptDownloads: false,
        permissions: [],
      });

      // 3. Set up hard session deadline timer (client timeout + buffer)
      const hardTimeoutMs = reqData.timeout_ms + this.hardTimeoutBufferMs;
      const deadlinePromise = new Promise((_, reject) => {
        deadlineTimer = setTimeout(() => {
          reject(
            new RenderEngineError(
              'SessionTimeout',
              `Hard rendering session deadline of ${hardTimeoutMs}ms exceeded.`,
              504
            )
          );
        }, hardTimeoutMs);
      });

      // 4. Race session execution against hard deadline
      const sessionPromise = this.executeRenderSession(context, reqData);
      return await Promise.race([sessionPromise, deadlinePromise]);
    } catch (err) {
      throw this.mapRenderError(err, reqData);
    } finally {
      if (deadlineTimer) {
        clearTimeout(deadlineTimer);
      }
      // Unconditionally close the isolated context to prevent process and RAM leaks
      if (context) {
        await context.close().catch(() => {});
      }
      // Release concurrency token
      this.limiter.release();
      this.rendersCount++;
      this.totalRendersServed++;

      // Check for memory/render count recycling threshold when idle
      if (
        this.limiter.activeCount === 0 &&
        (this.rendersCount >= this.maxRendersPerBrowser || process.memoryUsage().rss > this.maxMemoryRssBytes)
      ) {
        await this.recycleBrowser();
      }
    }
  }

  /**
   * Executes navigation, subresource filtering, error capturing, and DOM extraction.
   *
   * @private
   * @param {import('playwright-core').BrowserContext} context
   * @param {import('./protocol.js').RenderRequest} reqData
   * @returns {Promise<import('./protocol.js').RenderResponse>}
   */
  async executeRenderSession(context, reqData) {
    const page = await context.newPage();
    page.setDefaultNavigationTimeout(reqData.timeout_ms);
    page.setDefaultTimeout(reqData.timeout_ms);

    // Auto-dismiss dialogs (alert, confirm, prompt, beforeunload) so page never hangs
    page.on('dialog', async (dialog) => {
      await dialog.dismiss().catch(() => {});
    });

    const redirects = [];
    const consoleMessages = [];
    const pageErrors = [];
    const blockedSubresources = [];
    let subresourcesCount = 0;

    // Track HTTP redirect responses
    page.on('response', (response) => {
      const status = response.status();
      if (status >= 300 && status < 400) {
        const headers = response.headers();
        redirects.push({
          from_url: response.url(),
          to_url: headers.location || '',
          status_code: status,
        });
      }
    });

    // Capture browser console messages (bounded)
    page.on('console', (msg) => {
      if (consoleMessages.length < MAX_CONSOLE_MESSAGES) {
        const loc = msg.location ? msg.location() : null;
        consoleMessages.push({
          message_type: msg.type(),
          text: String(msg.text() || '').slice(0, 2000),
          location: loc && loc.url ? `${loc.url}:${loc.lineNumber || 0}` : null,
        });
      }
    });

    // Capture uncaught page exceptions (bounded)
    page.on('pageerror', (err) => {
      if (pageErrors.length < MAX_PAGE_ERRORS) {
        pageErrors.push({
          message: String(err.message || err).slice(0, 2000),
          stack: err.stack ? String(err.stack).slice(0, 4000) : null,
        });
      }
    });

    // Subresource filtering: max subresources limit & SSRF protection
    await page.route('**/*', (route) => {
      const req = route.request();
      const reqUrl = req.url();

      // Always allow data: and blob: inline assets
      if (reqUrl.startsWith('data:') || reqUrl.startsWith('blob:')) {
        return route.continue();
      }

      // Enforce max subresource ceiling per page
      if (subresourcesCount >= this.maxSubresources) {
        blockedSubresources.push({
          url: reqUrl,
          reason: `Subresource count exceeded maximum limit of ${this.maxSubresources}`,
          resource_type: req.resourceType(),
        });
        return route.abort('blockedbyclient');
      }

      try {
        const parsed = new URL(reqUrl);

        // Block unsupported or dangerous schemes
        if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
          blockedSubresources.push({
            url: reqUrl,
            reason: `Blocked unsupported scheme '${parsed.protocol}'`,
            resource_type: req.resourceType(),
          });
          return route.abort('blockedbyclient');
        }

        // SSRF protection: block private, loopback, CGNAT, link-local, and metadata destinations
        if (reqData.block_subresource_ips && this.isPrivateOrLoopbackHost(parsed.hostname)) {
          blockedSubresources.push({
            url: reqUrl,
            reason: `Blocked private/loopback destination: ${parsed.hostname}`,
            resource_type: req.resourceType(),
          });
          return route.abort('blockedbyclient');
        }

        subresourcesCount++;
        return route.continue();
      } catch {
        return route.abort('blockedbyclient');
      }
    });

    const navStart = Date.now();

    // Navigate to page
    const mainResponse = await page.goto(reqData.url, {
      waitUntil: reqData.wait_until,
      timeout: reqData.timeout_ms,
    });

    const domLoaded = Date.now();

    // Optional selector waiting
    if (reqData.wait_for_selector) {
      await page.waitForSelector(reqData.wait_for_selector, {
        timeout: reqData.timeout_ms,
      });
    }

    const pageLoaded = Date.now();

    // Extract rendered DOM and metadata
    const html = await page.content();
    const title = await page.title();
    const finalUrl = page.url();
    const statusCode = mainResponse ? mainResponse.status() : 200;

    // Enforce HTML payload size ceiling
    const htmlBytes = Buffer.byteLength(html, 'utf8');
    if (htmlBytes > this.maxHtmlBytes) {
      throw new RenderEngineError(
        'PayloadTooLarge',
        `Rendered HTML payload size (${htmlBytes} bytes) exceeds maximum ceiling of ${this.maxHtmlBytes} bytes`,
        413
      );
    }

    return createRenderResponse({
      url: finalUrl,
      status_code: statusCode,
      html,
      title,
      load_time_ms: pageLoaded - navStart,
      redirects,
      console_messages: consoleMessages,
      page_errors: pageErrors,
      blocked_subresources: blockedSubresources,
      timing: {
        navigation_start: navStart,
        dom_content_loaded: domLoaded,
        load: pageLoaded,
      },
    });
  }

  /**
   * Determines if a hostname is private, loopback, CGNAT, link-local, or cloud metadata.
   *
   * @param {string} rawHost
   * @returns {boolean}
   */
  isPrivateOrLoopbackHost(rawHost) {
    if (!rawHost || typeof rawHost !== 'string') {
      return true;
    }

    let host = rawHost.trim().toLowerCase();

    // Strip brackets if IPv6
    if (host.startsWith('[') && host.endsWith(']')) {
      host = host.slice(1, -1);
    }

    // 1. Hostname suffixes & names
    if (
      host === 'localhost' ||
      host.endsWith('.localhost') ||
      host === 'local' ||
      host.endsWith('.local') ||
      host.endsWith('.internal') ||
      host.endsWith('.test') ||
      host.endsWith('.example') ||
      host.endsWith('.invalid') ||
      host === 'metadata.google.internal' ||
      host === 'instance-data'
    ) {
      return true;
    }

    // 2. IPv6 Checks
    if (host.includes(':')) {
      // Loopback & Unspecified
      if (host === '::1' || host === '::' || host === '0:0:0:0:0:0:0:1' || host === '0:0:0:0:0:0:0:0') {
        return true;
      }
      // Link-local (fe80::/10 -> fe8, fe9, fea, feb)
      if (/^fe[89ab]/i.test(host)) {
        return true;
      }
      // Unique Local Address (fc00::/7 -> fc, fd)
      if (/^f[cd]/i.test(host)) {
        return true;
      }
      // IPv4-mapped IPv6 (::ffff:x.x.x.x or ::ffff:hex)
      if (host.startsWith('::ffff:')) {
        const remainder = host.slice(7);
        if (remainder.includes('.')) {
          return this.isPrivateOrLoopbackHost(remainder);
        }
        // Hex representation (e.g. 7f00:1)
        const hexParts = remainder.split(':');
        if (hexParts.length === 2) {
          const high = Number.parseInt(hexParts[0], 16);
          const low = Number.parseInt(hexParts[1], 16);
          const ip = `${(high >> 8) & 0xff}.${high & 0xff}.${(low >> 8) & 0xff}.${low & 0xff}`;
          return this.isPrivateOrLoopbackHost(ip);
        }
        return true;
      }
      return false;
    }

    // 3. IPv4 Checks
    const ipv4Parts = host.split('.');
    if (ipv4Parts.length === 4 && ipv4Parts.every((p) => /^\d+$/.test(p))) {
      const [o1, o2, o3, o4] = ipv4Parts.map((p) => Number.parseInt(p, 10));
      if ([o1, o2, o3, o4].some((o) => o < 0 || o > 255)) {
        return true;
      }

      // 0.0.0.0/8 (Current network)
      if (o1 === 0) return true;

      // 10.0.0.0/8 (Private RFC 1918)
      if (o1 === 10) return true;

      // 127.0.0.0/8 (Loopback)
      if (o1 === 127) return true;

      // 169.254.0.0/16 (Link-local / Cloud metadata)
      if (o1 === 169 && o2 === 254) return true;

      // 172.16.0.0/12 (Private RFC 1918)
      if (o1 === 172 && o2 >= 16 && o2 <= 31) return true;

      // 192.168.0.0/16 (Private RFC 1918)
      if (o1 === 192 && o2 === 168) return true;

      // 100.64.0.0/10 (Carrier-Grade NAT)
      if (o1 === 100 && o2 >= 64 && o2 <= 127) return true;

      // 100.100.100.200 (Alibaba Cloud metadata)
      if (o1 === 100 && o2 === 100 && o3 === 100 && o4 === 200) return true;

      // 198.18.0.0/15 (Benchmarking)
      if (o1 === 198 && (o2 === 18 || o2 === 19)) return true;

      // 224.0.0.0/4 (Multicast) & 240.0.0.0/4 (Reserved) & 255.255.255.255
      if (o1 >= 224) return true;

      return false;
    }

    return false;
  }

  /**
   * Maps Playwright and network exceptions to RenderEngineError.
   *
   * @private
   * @param {Error} err
   * @param {import('./protocol.js').RenderRequest} reqData
   * @returns {RenderEngineError}
   */
  mapRenderError(err, reqData) {
    if (err instanceof RenderEngineError) {
      return err;
    }

    const msg = err.message || String(err);

    if (err.name === 'TimeoutError' || msg.includes('Timeout') || msg.includes('timeout')) {
      if (msg.includes('waitForSelector') || msg.includes('waiting for selector')) {
        return new RenderEngineError(
          'SelectorTimeout',
          `Timed out waiting for selector '${reqData.wait_for_selector}': ${msg}`,
          504
        );
      }
      return new RenderEngineError(
        'NavigationTimeout',
        `Timed out after ${reqData.timeout_ms}ms during navigation to '${reqData.url}': ${msg}`,
        504
      );
    }

    if (msg.includes('net::ERR_NAME_NOT_RESOLVED')) {
      return new RenderEngineError(
        'DnsResolutionFailed',
        `DNS resolution failed for '${reqData.url}': ${msg}`,
        502
      );
    }

    if (msg.includes('net::ERR_CONNECTION_REFUSED')) {
      return new RenderEngineError(
        'ConnectionRefused',
        `Connection refused by host for '${reqData.url}': ${msg}`,
        502
      );
    }

    if (msg.includes('net::ERR_CERT_') || msg.includes('SSL') || msg.includes('certificate')) {
      return new RenderEngineError(
        'TlsCertificateError',
        `TLS/SSL certificate error for '${reqData.url}': ${msg}`,
        502
      );
    }

    return new RenderEngineError(
      'RenderExecutionError',
      `Dynamic page rendering failed: ${msg}`,
      500
    );
  }

  /**
   * Returns current engine performance, queue, and memory stats.
   *
   * @returns {object}
   */
  getStats() {
    return {
      concurrency: this.limiter.getStats(),
      renders_count: this.rendersCount,
      total_renders_served: this.totalRendersServed,
      memory_rss_bytes: process.memoryUsage().rss,
    };
  }

  /**
   * Closes browser instance and all active contexts.
   *
   * @returns {Promise<void>}
   */
  async close() {
    this.closed = true;
    if (this.browser) {
      try {
        await this.browser.close();
      } catch {
        // Ignored during shutdown
      }
      this.browser = null;
    }
  }
}

// Global default singleton instance
let defaultEngine = null;

/**
 * Returns the shared PlaywrightEngine instance.
 *
 * @returns {PlaywrightEngine}
 */
export function getBrowserEngine() {
  if (!defaultEngine) {
    defaultEngine = new PlaywrightEngine();
  }
  return defaultEngine;
}
