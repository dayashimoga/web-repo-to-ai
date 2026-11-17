/**
 * @file protocol.js
 * @description Strongly typed IPC/RPC protocol schemas, validation, and error envelopes
 * for the isolated browser rendering worker.
 */

export const DEFAULT_VIEWPORT = { width: 1280, height: 800 };
export const DEFAULT_TIMEOUT_MS = 30000;
export const MIN_TIMEOUT_MS = 1000;
export const MAX_TIMEOUT_MS = 60000;
export const ALLOWED_WAIT_UNTIL = ['domcontentloaded', 'load', 'networkidle'];

/**
 * Validates and normalizes an incoming render request payload.
 *
 * @param {unknown} body - Raw parsed JSON request body.
 * @returns {{ valid: true, data: RenderRequest } | { valid: false, error: ProblemDetails }}
 */
export function validateRenderRequest(body) {
  if (!body || typeof body !== 'object' || Array.isArray(body)) {
    return {
      valid: false,
      error: createProblemDetails(
        'InvalidRequest',
        'Request body must be a non-null JSON object.',
        400
      ),
    };
  }

  // 1. URL validation
  if (typeof body.url !== 'string' || body.url.trim().length === 0) {
    return {
      valid: false,
      error: createProblemDetails(
        'InvalidUrl',
        "Field 'url' is required and must be a non-empty string.",
        400
      ),
    };
  }

  let parsedUrl;
  try {
    parsedUrl = new URL(body.url.trim());
  } catch (err) {
    return {
      valid: false,
      error: createProblemDetails(
        'InvalidUrl',
        `Field 'url' is not a valid absolute URL: ${err.message}`,
        400
      ),
    };
  }

  if (parsedUrl.protocol !== 'http:' && parsedUrl.protocol !== 'https:') {
    return {
      valid: false,
      error: createProblemDetails(
        'InvalidScheme',
        `URL scheme '${parsedUrl.protocol}' is not supported. Only 'http:' and 'https:' are allowed.`,
        400
      ),
    };
  }

  // 2. wait_until validation
  let waitUntil = 'networkidle';
  if (body.wait_until !== undefined) {
    if (typeof body.wait_until !== 'string' || !ALLOWED_WAIT_UNTIL.includes(body.wait_until.toLowerCase())) {
      return {
        valid: false,
        error: createProblemDetails(
          'InvalidOption',
          `Field 'wait_until' must be one of: ${ALLOWED_WAIT_UNTIL.join(', ')}.`,
          400
        ),
      };
    }
    waitUntil = body.wait_until.toLowerCase();
  }

  // 3. timeout_ms validation
  let timeoutMs = DEFAULT_TIMEOUT_MS;
  if (body.timeout_ms !== undefined) {
    if (typeof body.timeout_ms !== 'number' || !Number.isInteger(body.timeout_ms) || body.timeout_ms < MIN_TIMEOUT_MS || body.timeout_ms > MAX_TIMEOUT_MS) {
      return {
        valid: false,
        error: createProblemDetails(
          'InvalidOption',
          `Field 'timeout_ms' must be an integer between ${MIN_TIMEOUT_MS} and ${MAX_TIMEOUT_MS}.`,
          400
        ),
      };
    }
    timeoutMs = body.timeout_ms;
  }

  // 4. wait_for_selector validation
  let waitForSelector = null;
  if (body.wait_for_selector !== undefined && body.wait_for_selector !== null) {
    if (typeof body.wait_for_selector !== 'string' || body.wait_for_selector.trim().length === 0) {
      return {
        valid: false,
        error: createProblemDetails(
          'InvalidOption',
          "Field 'wait_for_selector' must be a non-empty string or null.",
          400
        ),
      };
    }
    waitForSelector = body.wait_for_selector.trim();
  }

  // 5. viewport validation
  let viewport = { ...DEFAULT_VIEWPORT };
  if (body.viewport !== undefined && body.viewport !== null) {
    if (typeof body.viewport !== 'object' || Array.isArray(body.viewport)) {
      return {
        valid: false,
        error: createProblemDetails('InvalidOption', "Field 'viewport' must be an object with width and height.", 400),
      };
    }
    const { width, height } = body.viewport;
    if (typeof width !== 'number' || width < 320 || width > 3840 || typeof height !== 'number' || height < 240 || height > 2160) {
      return {
        valid: false,
        error: createProblemDetails(
          'InvalidOption',
          "Viewport 'width' must be 320..3840 and 'height' must be 240..2160.",
          400
        ),
      };
    }
    viewport = { width: Math.floor(width), height: Math.floor(height) };
  }

  // 6. user_agent validation
  let userAgent = null;
  if (body.user_agent !== undefined && body.user_agent !== null) {
    if (typeof body.user_agent !== 'string') {
      return {
        valid: false,
        error: createProblemDetails('InvalidOption', "Field 'user_agent' must be a string or null.", 400),
      };
    }
    userAgent = body.user_agent.trim() || null;
  }

  // 7. headers validation
  let headers = {};
  if (body.headers !== undefined && body.headers !== null) {
    if (typeof body.headers !== 'object' || Array.isArray(body.headers)) {
      return {
        valid: false,
        error: createProblemDetails('InvalidOption', "Field 'headers' must be an object of key-value string pairs.", 400),
      };
    }
    for (const [k, v] of Object.entries(body.headers)) {
      if (typeof v !== 'string') {
        return {
          valid: false,
          error: createProblemDetails('InvalidOption', `Header value for '${k}' must be a string.`, 400),
        };
      }
      headers[k] = v;
    }
  }

  // 8. boolean flags
  const javascriptEnabled = body.javascript_enabled !== false;
  const blockSubresourceIps = body.block_subresource_ips !== false;

  return {
    valid: true,
    data: {
      url: parsedUrl.toString(),
      wait_until: waitUntil,
      timeout_ms: timeoutMs,
      wait_for_selector: waitForSelector,
      viewport,
      user_agent: userAgent,
      headers,
      javascript_enabled: javascriptEnabled,
      block_subresource_ips: blockSubresourceIps,
    },
  };
}

/**
 * Creates an RFC 7807 compatible problem details error envelope.
 *
 * @param {string} errorCode - Machine-readable error code.
 * @param {string} detail - Human-readable explanation.
 * @param {number} statusCode - HTTP status code.
 * @returns {ProblemDetails}
 */
export function createProblemDetails(errorCode, detail, statusCode = 500) {
  return {
    type: `https://web-repo-to-ai.dev/errors/${errorCode.toLowerCase()}`,
    title: errorCode,
    status: statusCode,
    detail,
    error_code: errorCode,
    timestamp: new Date().toISOString(),
  };
}

/**
 * Creates a standard render response payload.
 *
 * @param {Partial<RenderResponse>} data
 * @returns {RenderResponse}
 */
export function createRenderResponse(data) {
  return {
    url: data.url || '',
    status_code: data.status_code || 200,
    html: data.html || '',
    title: data.title || '',
    load_time_ms: data.load_time_ms || 0,
    redirects: data.redirects || [],
    console_messages: data.console_messages || [],
    page_errors: data.page_errors || [],
    blocked_subresources: data.blocked_subresources || [],
    timing: data.timing || {
      navigation_start: Date.now(),
      dom_content_loaded: Date.now(),
      load: Date.now(),
    },
  };
}
