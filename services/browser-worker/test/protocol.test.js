import { test, describe, before, after } from 'node:test';
import assert from 'node:assert';
import http from 'node:http';
import { validateRenderRequest, createProblemDetails, createRenderResponse } from '../src/protocol.js';
import { createServer } from '../server.js';

describe('Browser Worker IPC Protocol & Schema Validation', () => {
  test('validates valid render request with defaults', () => {
    const result = validateRenderRequest({ url: 'https://example.com/app' });
    assert.strictEqual(result.valid, true);
    assert.strictEqual(result.data.url, 'https://example.com/app');
    assert.strictEqual(result.data.wait_until, 'networkidle');
    assert.strictEqual(result.data.timeout_ms, 30000);
    assert.strictEqual(result.data.viewport.width, 1280);
    assert.strictEqual(result.data.viewport.height, 800);
    assert.strictEqual(result.data.javascript_enabled, true);
    assert.strictEqual(result.data.block_subresource_ips, true);
  });

  test('validates custom options within allowed bounds', () => {
    const result = validateRenderRequest({
      url: 'http://docs.example.org:8080/page',
      wait_until: 'domcontentloaded',
      timeout_ms: 15000,
      wait_for_selector: '#content',
      viewport: { width: 1920, height: 1080 },
      user_agent: 'CustomBot/1.0',
      headers: { 'X-Custom-Header': 'value' },
      javascript_enabled: false,
      block_subresource_ips: false,
    });

    assert.strictEqual(result.valid, true);
    assert.strictEqual(result.data.wait_until, 'domcontentloaded');
    assert.strictEqual(result.data.timeout_ms, 15000);
    assert.strictEqual(result.data.wait_for_selector, '#content');
    assert.strictEqual(result.data.viewport.width, 1920);
    assert.strictEqual(result.data.viewport.height, 1080);
    assert.strictEqual(result.data.user_agent, 'CustomBot/1.0');
    assert.strictEqual(result.data.headers['X-Custom-Header'], 'value');
    assert.strictEqual(result.data.javascript_enabled, false);
    assert.strictEqual(result.data.block_subresource_ips, false);
  });

  test('rejects missing or empty url', () => {
    const res1 = validateRenderRequest({});
    assert.strictEqual(res1.valid, false);
    assert.strictEqual(res1.error.error_code, 'InvalidUrl');

    const res2 = validateRenderRequest({ url: '   ' });
    assert.strictEqual(res2.valid, false);
    assert.strictEqual(res2.error.error_code, 'InvalidUrl');
  });

  test('rejects unsupported schemes (file://, javascript:, ftp://)', () => {
    const res1 = validateRenderRequest({ url: 'file:///etc/passwd' });
    assert.strictEqual(res1.valid, false);
    assert.strictEqual(res1.error.error_code, 'InvalidScheme');

    const res2 = validateRenderRequest({ url: 'javascript:alert(1)' });
    assert.strictEqual(res2.valid, false);
    assert.strictEqual(res2.error.error_code, 'InvalidScheme');

    const res3 = validateRenderRequest({ url: 'ftp://ftp.example.com/file' });
    assert.strictEqual(res3.valid, false);
    assert.strictEqual(res3.error.error_code, 'InvalidScheme');
  });

  test('rejects invalid timeout_ms values', () => {
    const res1 = validateRenderRequest({ url: 'https://example.com', timeout_ms: 100 });
    assert.strictEqual(res1.valid, false);
    assert.strictEqual(res1.error.error_code, 'InvalidOption');

    const res2 = validateRenderRequest({ url: 'https://example.com', timeout_ms: 120000 });
    assert.strictEqual(res2.valid, false);
    assert.strictEqual(res2.error.error_code, 'InvalidOption');
  });

  test('creates RFC 7807 problem details envelope', () => {
    const problem = createProblemDetails('BlockedHost', 'Target IP is within loopback range.', 403);
    assert.strictEqual(problem.title, 'BlockedHost');
    assert.strictEqual(problem.status, 403);
    assert.strictEqual(problem.error_code, 'BlockedHost');
    assert.ok(problem.type.includes('blockedhost'));
    assert.ok(problem.timestamp);
  });

  test('creates standard render response envelope', () => {
    const resp = createRenderResponse({
      url: 'https://example.com',
      html: '<html><body>Hello</body></html>',
      title: 'Example',
      load_time_ms: 120,
    });
    assert.strictEqual(resp.url, 'https://example.com');
    assert.strictEqual(resp.status_code, 200);
    assert.strictEqual(resp.title, 'Example');
    assert.strictEqual(resp.load_time_ms, 120);
    assert.ok(Array.isArray(resp.redirects));
    assert.ok(Array.isArray(resp.console_messages));
    assert.ok(Array.isArray(resp.page_errors));
    assert.ok(Array.isArray(resp.blocked_subresources));
    assert.ok(resp.timing.navigation_start);
  });
});

describe('Browser Worker HTTP Server Integration', () => {
  let server;
  let port;

  before(async () => {
    const testEngine = {
      isReady: async () => true,
      render: async (reqData) => {
        return createRenderResponse({
          url: reqData.url,
          status_code: 200,
          html: `<!DOCTYPE html><html><head><title>Test Page</title></head><body><h1>Rendered</h1><p>Rendered URL: ${reqData.url}</p></body></html>`,
          title: 'Test Page',
          load_time_ms: 25,
        });
      },
      close: async () => {},
    };
    server = createServer({ browserEngine: testEngine });
    await new Promise((resolve) => {
      server.listen(0, '127.0.0.1', () => {
        port = server.address().port;
        resolve();
      });
    });
  });

  after(async () => {
    await new Promise((resolve) => server.close(resolve));
  });

  async function request(path, options = {}) {
    return new Promise((resolve, reject) => {
      const req = http.request(
        {
          hostname: '127.0.0.1',
          port,
          path,
          method: options.method || 'GET',
          headers: options.headers || {},
        },
        (res) => {
          let chunks = [];
          res.on('data', (c) => chunks.push(c));
          res.on('end', () => {
            const bodyStr = Buffer.concat(chunks).toString('utf8');
            let json = null;
            try {
              json = JSON.parse(bodyStr);
            } catch {
              // Not JSON
            }
            resolve({
              status: res.statusCode,
              headers: res.headers,
              body: bodyStr,
              json,
            });
          });
        }
      );
      req.on('error', reject);
      if (options.body) {
        req.write(options.body);
      }
      req.end();
    });
  }

  test('GET /health returns 200 with service metadata', async () => {
    const res = await request('/health');
    assert.strictEqual(res.status, 200);
    assert.strictEqual(res.json.status, 'ok');
    assert.strictEqual(res.json.service, 'browser-worker');
    assert.strictEqual(res.json.version, '0.1.0');
    assert.ok(res.json.timestamp);
  });

  test('GET /ready returns 200 with readiness indicator', async () => {
    const res = await request('/ready');
    assert.strictEqual(res.status, 200);
    assert.strictEqual(res.json.status, 'ready');
    assert.strictEqual(res.json.browser_ready, true);
  });

  test('POST /render with valid payload returns 200 and protocol response', async () => {
    const res = await request('/render', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        url: 'https://news.ycombinator.com',
        wait_until: 'load',
        timeout_ms: 10000,
      }),
    });

    assert.strictEqual(res.status, 200);
    assert.strictEqual(res.headers['content-type'], 'application/json');
    assert.strictEqual(res.json.url, 'https://news.ycombinator.com/');
    assert.strictEqual(res.json.status_code, 200);
    assert.ok(res.json.html.includes('Test Page'));
    assert.ok(res.json.load_time_ms >= 0);
  });

  test('POST /render with invalid scheme returns 400 with problem details', async () => {
    const res = await request('/render', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        url: 'file:///etc/hosts',
      }),
    });

    assert.strictEqual(res.status, 400);
    assert.strictEqual(res.headers['content-type'], 'application/problem+json');
    assert.strictEqual(res.json.error_code, 'InvalidScheme');
    assert.strictEqual(res.json.status, 400);
  });

  test('GET /render returns 405 Method Not Allowed', async () => {
    const res = await request('/render');
    assert.strictEqual(res.status, 405);
    assert.strictEqual(res.json.error_code, 'MethodNotAllowed');
  });

  test('GET /unknown-route returns 404', async () => {
    const res = await request('/unknown-route');
    assert.strictEqual(res.status, 404);
    assert.strictEqual(res.json.error_code, 'NotFound');
  });
});
