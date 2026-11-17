/**
 * @file routes.js
 * @description HTTP request dispatcher and route handlers for the browser worker service.
 */

import { validateRenderRequest, createProblemDetails } from './protocol.js';
import { getBrowserEngine, RenderEngineError } from './engine.js';

export const SERVICE_NAME = 'browser-worker';
export const SERVICE_VERSION = '0.1.0';

/**
 * Dispatches incoming HTTP requests to corresponding endpoint handlers.
 *
 * @param {import('node:http').IncomingMessage} req
 * @param {import('node:http').ServerResponse} res
 * @param {object} [options={}]
 * @param {import('./engine.js').PlaywrightEngine} [options.browserEngine]
 */
export async function handleRequest(req, res, options = {}) {
  const url = new URL(req.url || '/', `http://${req.headers.host || 'localhost'}`);
  const path = url.pathname;
  const method = req.method ? req.method.toUpperCase() : 'GET';
  const engine = options.browserEngine || getBrowserEngine();

  // Liveness check
  if ((path === '/health' || path === '/live') && method === 'GET') {
    const stats = typeof engine.getStats === 'function' ? engine.getStats() : undefined;
    return sendJson(res, 200, {
      status: 'ok',
      service: SERVICE_NAME,
      version: SERVICE_VERSION,
      stats,
      timestamp: new Date().toISOString(),
    });
  }

  // Readiness check
  if (path === '/ready' && method === 'GET') {
    const isReady = await engine.isReady();
    const statusCode = isReady ? 200 : 503;
    const stats = typeof engine.getStats === 'function' ? engine.getStats() : undefined;
    return sendJson(res, statusCode, {
      status: isReady ? 'ready' : 'not_ready',
      service: SERVICE_NAME,
      version: SERVICE_VERSION,
      browser_ready: isReady,
      stats,
      timestamp: new Date().toISOString(),
    });
  }

  // Render endpoint
  if (path === '/render' || path === '/api/v1/render') {
    if (method !== 'POST') {
      return sendProblem(res, 405, 'MethodNotAllowed', `HTTP method '${method}' not allowed on '${path}'.`);
    }

    try {
      const rawBody = await readBody(req);
      let parsedBody;
      try {
        parsedBody = rawBody.length > 0 ? JSON.parse(rawBody) : {};
      } catch (jsonErr) {
        return sendProblem(res, 400, 'MalformedJson', `Malformed JSON request body: ${jsonErr.message}`);
      }

      const validation = validateRenderRequest(parsedBody);
      if (!validation.valid) {
        return sendJson(res, validation.error.status, validation.error, 'application/problem+json');
      }

      const reqData = validation.data;

      // Execute dynamic rendering via PlaywrightEngine
      try {
        const responseData = await engine.render(reqData);
        return sendJson(res, 200, responseData);
      } catch (renderErr) {
        if (renderErr instanceof RenderEngineError) {
          const extraHeaders = renderErr.statusCode === 429 ? { 'Retry-After': '5' } : {};
          return sendProblem(res, renderErr.statusCode, renderErr.code, renderErr.detail, extraHeaders);
        }
        return sendProblem(res, 500, 'RenderExecutionError', renderErr.message || String(renderErr));
      }
    } catch (err) {
      return sendProblem(res, 500, 'InternalError', `Unexpected internal server error: ${err.message}`);
    }
  }

  return sendProblem(res, 404, 'NotFound', `Route '${path}' was not found.`);
}

/**
 * Reads request stream into a UTF-8 string with byte limit (1 MB).
 *
 * @param {import('node:http').IncomingMessage} req
 * @returns {Promise<string>}
 */
function readBody(req) {
  return new Promise((resolve, reject) => {
    let chunks = [];
    let bytes = 0;
    const maxBytes = 1024 * 1024; // 1 MB payload limit

    req.on('data', (chunk) => {
      bytes += chunk.length;
      if (bytes > maxBytes) {
        req.destroy();
        reject(new Error('Payload too large: request body exceeds 1 MB'));
        return;
      }
      chunks.push(chunk);
    });

    req.on('end', () => {
      resolve(Buffer.concat(chunks).toString('utf8'));
    });

    req.on('error', (err) => {
      reject(err);
    });
  });
}

/**
 * Sends a JSON response with status code and headers.
 *
 * @param {import('node:http').ServerResponse} res
 * @param {number} status
 * @param {unknown} data
 * @param {string} [contentType='application/json']
 * @param {Record<string, string>} [extraHeaders={}]
 */
function sendJson(res, status, data, contentType = 'application/json', extraHeaders = {}) {
  const body = JSON.stringify(data);
  res.writeHead(status, {
    'Content-Type': contentType,
    'Content-Length': Buffer.byteLength(body),
    'Connection': 'keep-alive',
    ...extraHeaders,
  });
  res.end(body);
}

/**
 * Sends an RFC 7807 problem details response.
 *
 * @param {import('node:http').ServerResponse} res
 * @param {number} status
 * @param {string} code
 * @param {string} detail
 * @param {Record<string, string>} [extraHeaders={}]
 */
function sendProblem(res, status, code, detail, extraHeaders = {}) {
  const problem = createProblemDetails(code, detail, status);
  sendJson(res, status, problem, 'application/problem+json', extraHeaders);
}
