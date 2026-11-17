/**
 * @file routes.js
 * @description HTTP request dispatcher and route handlers for the browser worker service.
 */

import { validateRenderRequest, createProblemDetails, createRenderResponse } from './protocol.js';

export const SERVICE_NAME = 'browser-worker';
export const SERVICE_VERSION = '0.1.0';

/**
 * Dispatches incoming HTTP requests to corresponding endpoint handlers.
 *
 * @param {import('node:http').IncomingMessage} req
 * @param {import('node:http').ServerResponse} res
 */
export async function handleRequest(req, res) {
  const url = new URL(req.url || '/', `http://${req.headers.host || 'localhost'}`);
  const path = url.pathname;
  const method = req.method ? req.method.toUpperCase() : 'GET';

  // Liveness check
  if ((path === '/health' || path === '/live') && method === 'GET') {
    return sendJson(res, 200, {
      status: 'ok',
      service: SERVICE_NAME,
      version: SERVICE_VERSION,
      timestamp: new Date().toISOString(),
    });
  }

  // Readiness check
  if (path === '/ready' && method === 'GET') {
    return sendJson(res, 200, {
      status: 'ready',
      service: SERVICE_NAME,
      version: SERVICE_VERSION,
      browser_ready: true,
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

      // In Subphase 3.3.1 (Architecture & Protocol baseline), return structured IPC response envelope.
      // Full Playwright browser page rendering integration takes place in Subphase 3.3.2.
      const responseData = createRenderResponse({
        url: reqData.url,
        status_code: 200,
        html: `<!DOCTYPE html><html><head><title>Rendered</title></head><body><div id="root"><h1>Browser Worker Active</h1><p>Rendered URL: ${reqData.url}</p></div></body></html>`,
        title: 'Browser Worker Active',
        load_time_ms: 50,
        timing: {
          navigation_start: Date.now() - 50,
          dom_content_loaded: Date.now() - 20,
          load: Date.now(),
        },
      });

      return sendJson(res, 200, responseData);
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
 * @param {string} contentType
 */
function sendJson(res, status, data, contentType = 'application/json') {
  const body = JSON.stringify(data);
  res.writeHead(status, {
    'Content-Type': contentType,
    'Content-Length': Buffer.byteLength(body),
    'Connection': 'keep-alive',
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
 */
function sendProblem(res, status, code, detail) {
  const problem = createProblemDetails(code, detail, status);
  sendJson(res, status, problem, 'application/problem+json');
}
