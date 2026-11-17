/**
 * @file engine.js
 * @description Playwright / Chromium dynamic browser rendering engine with lifecycle
 * management, per-request context isolation, and SSRF subresource interception.
 */

import { chromium } from 'playwright-core';
import { createRenderResponse } from './protocol.js';

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

/**
 * Custom error class representing rendering execution and timeout failures.
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
 * Manages Chromium browser lifecycle and isolated page rendering sessions.
 */
export class PlaywrightEngine {
  /**
   * @param {object} [options={}]
   * @param {typeof chromium} [options.chromiumModule] - Dependency injection for testing.
   * @param {string[]} [options.args] - Custom Chromium CLI arguments.
   * @param {string} [options.executablePath] - Custom browser executable path.
   */
  constructor(options = {}) {
    this.chromiumModule = options.chromiumModule || chromium;
    this.args = options.args || DEFAULT_CHROMIUM_ARGS;
    this.executablePath = options.executablePath || process.env.CHROME_BIN || process.env.BROWSER_PATH || null;
    this.browser = null;
    this.launchPromise = null;
    this.closed = false;
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
      // If explicit path was provided or browser not found, attempt channel fallbacks on developer hosts
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
            // Throw original error with helpful guidance
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
   * Probes whether the browser is currently launched and ready to serve requests.
   *
   * @returns {Promise<boolean>}
   */
  async isReady() {
    if (this.closed) {
      return false;
    }
    try {
      const browser = await this.getBrowser();
      return browser && browser.isConnected();
    } catch {
      return false;
    }
  }

  /**
   * Renders a webpage inside a completely isolated BrowserContext and extracts the DOM.
   *
   * @param {import('./protocol.js').RenderRequest} reqData - Validated render request parameters.
   * @returns {Promise<import('./protocol.js').RenderResponse>}
   */
  async render(reqData) {
    const browser = await this.getBrowser();

    // Context isolation: each render request receives a fresh, isolated BrowserContext
    const context = await browser.newContext({
      viewport: reqData.viewport,
      userAgent: reqData.user_agent || undefined,
      extraHTTPHeaders: reqData.headers && Object.keys(reqData.headers).length > 0 ? reqData.headers : undefined,
      javaScriptEnabled: reqData.javascript_enabled !== false,
      ignoreHTTPSErrors: false,
    });

    try {
      return await this.executeRenderSession(context, reqData);
    } catch (err) {
      throw this.mapRenderError(err, reqData);
    } finally {
      // Unconditionally close the isolated context to prevent process and RAM leaks
      await context.close().catch(() => {});
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

    const redirects = [];
    const consoleMessages = [];
    const pageErrors = [];
    const blockedSubresources = [];

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

    // Subresource SSRF protection: block non-HTTP and private destinations
    if (reqData.block_subresource_ips) {
      await page.route('**/*', (route) => {
        const req = route.request();
        const reqUrl = req.url();

        // Always allow data: and blob: inline assets
        if (reqUrl.startsWith('data:') || reqUrl.startsWith('blob:')) {
          return route.continue();
        }

        try {
          const parsed = new URL(reqUrl);

          // Block unsupported schemes
          if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
            blockedSubresources.push({
              url: reqUrl,
              reason: `Blocked unsupported scheme '${parsed.protocol}'`,
              resource_type: req.resourceType(),
            });
            return route.abort('blockedbyclient');
          }

          // Block private / loopback / cloud metadata IP destinations
          const host = parsed.hostname.toLowerCase();
          if (this.isPrivateOrLoopbackHost(host)) {
            blockedSubresources.push({
              url: reqUrl,
              reason: `Blocked private/loopback destination: ${host}`,
              resource_type: req.resourceType(),
            });
            return route.abort('blockedbyclient');
          }
        } catch {
          return route.abort('blockedbyclient');
        }

        return route.continue();
      });
    }

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
   * Determines if a hostname is private, loopback, or cloud metadata.
   *
   * @param {string} host
   * @returns {boolean}
   */
  isPrivateOrLoopbackHost(host) {
    if (
      host === 'localhost' ||
      host === '127.0.0.1' ||
      host === '::1' ||
      host === '169.254.169.254' ||
      host.endsWith('.localhost') ||
      host.endsWith('.local')
    ) {
      return true;
    }

    if (host.startsWith('10.') || host.startsWith('192.168.')) {
      return true;
    }

    // 172.16.0.0/12
    const match172 = host.match(/^172\.(\d+)\./);
    if (match172) {
      const octet = Number.parseInt(match172[1], 10);
      if (octet >= 16 && octet <= 31) {
        return true;
      }
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
