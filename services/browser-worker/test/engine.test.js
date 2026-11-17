import { test, describe, before, after } from 'node:test';
import assert from 'node:assert';
import http from 'node:http';
import { PlaywrightEngine, RenderEngineError } from '../src/engine.js';
import { createServer } from '../server.js';

describe('PlaywrightEngine Unit & Lifecycle Tests', () => {
  const engine = new PlaywrightEngine();

  test('isPrivateOrLoopbackHost correctly classifies addresses', () => {
    // Loopback & local
    assert.strictEqual(engine.isPrivateOrLoopbackHost('localhost'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('127.0.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('::1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('app.localhost'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('myservice.local'), true);

    // Cloud metadata
    assert.strictEqual(engine.isPrivateOrLoopbackHost('169.254.169.254'), true);

    // Private IPv4 ranges (RFC 1918)
    assert.strictEqual(engine.isPrivateOrLoopbackHost('10.0.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('10.255.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('192.168.1.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('192.168.0.254'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('172.16.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('172.31.255.255'), true);

    // Public / Non-private
    assert.strictEqual(engine.isPrivateOrLoopbackHost('example.com'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('github.com'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('172.15.0.1'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('172.32.0.1'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('8.8.8.8'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('1.1.1.1'), false);
  });

  test('mapRenderError classifies navigation and selector timeouts', () => {
    const navTimeoutErr = new Error('page.goto: Timeout 30000ms exceeded.');
    navTimeoutErr.name = 'TimeoutError';
    const mappedNav = engine.mapRenderError(navTimeoutErr, { url: 'https://example.com', timeout_ms: 30000 });
    assert.strictEqual(mappedNav.code, 'NavigationTimeout');
    assert.strictEqual(mappedNav.statusCode, 504);

    const selTimeoutErr = new Error('page.waitForSelector: Timeout 5000ms exceeded waiting for selector "#missing"');
    selTimeoutErr.name = 'TimeoutError';
    const mappedSel = engine.mapRenderError(selTimeoutErr, {
      url: 'https://example.com',
      wait_for_selector: '#missing',
      timeout_ms: 5000,
    });
    assert.strictEqual(mappedSel.code, 'SelectorTimeout');
    assert.strictEqual(mappedSel.statusCode, 504);
  });

  test('mapRenderError classifies network errors', () => {
    const dnsErr = new Error('net::ERR_NAME_NOT_RESOLVED at https://nonexistent.invalid');
    const mappedDns = engine.mapRenderError(dnsErr, { url: 'https://nonexistent.invalid' });
    assert.strictEqual(mappedDns.code, 'DnsResolutionFailed');
    assert.strictEqual(mappedDns.statusCode, 502);

    const connRefusedErr = new Error('net::ERR_CONNECTION_REFUSED at http://127.0.0.1:9999');
    const mappedConn = engine.mapRenderError(connRefusedErr, { url: 'http://127.0.0.1:9999' });
    assert.strictEqual(mappedConn.code, 'ConnectionRefused');
    assert.strictEqual(mappedConn.statusCode, 502);

    const sslErr = new Error('net::ERR_CERT_COMMON_NAME_INVALID at https://badcert.com');
    const mappedSsl = engine.mapRenderError(sslErr, { url: 'https://badcert.com' });
    assert.strictEqual(mappedSsl.code, 'TlsCertificateError');
    assert.strictEqual(mappedSsl.statusCode, 502);

    const generalErr = new Error('Unexpected crash');
    const mappedGen = engine.mapRenderError(generalErr, { url: 'https://example.com' });
    assert.strictEqual(mappedGen.code, 'RenderExecutionError');
    assert.strictEqual(mappedGen.statusCode, 500);
  });
});

describe('Live Playwright Chromium Dynamic Rendering Integration', () => {
  let fixtureServer;
  let fixturePort;
  let workerServer;
  let workerPort;
  let engine;

  before(async () => {
    // 1. Start a local HTTP fixture server serving dynamic JS-rendered HTML
    fixtureServer = http.createServer((req, res) => {
      const url = new URL(req.url, `http://127.0.0.1:${fixturePort}`);

      if (url.pathname === '/dynamic') {
        res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
        res.end(`<!DOCTYPE html>
<html>
  <head>
    <title>Dynamic SPA Test</title>
  </head>
  <body>
    <div id="app">
      <div id="loader">Loading dynamic content...</div>
    </div>
    <script>
      // Simulate client-side JavaScript DOM hydration
      setTimeout(() => {
        const app = document.getElementById('app');
        app.innerHTML = '<h1 id="rendered-title">Hydrated Content Ready</h1><p id="desc">Extracted via Playwright</p>';
        console.log('[SPA] Hydration finished successfully');
      }, 50);
    </script>
  </body>
</html>`);
        return;
      }

      if (url.pathname === '/redirect-target') {
        res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
        res.end('<!DOCTYPE html><html><head><title>Destination Page</title></head><body><h1>Arrived</h1></body></html>');
        return;
      }

      if (url.pathname === '/redirect-source') {
        res.writeHead(302, { Location: `http://127.0.0.1:${fixturePort}/redirect-target` });
        res.end();
        return;
      }

      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end('Not found');
    });

    await new Promise((resolve) => {
      fixtureServer.listen(0, '127.0.0.1', () => {
        fixturePort = fixtureServer.address().port;
        resolve();
      });
    });

    // 2. Start browser worker server with real PlaywrightEngine
    engine = new PlaywrightEngine();
    workerServer = createServer({ browserEngine: engine });
    await new Promise((resolve) => {
      workerServer.listen(0, '127.0.0.1', () => {
        workerPort = workerServer.address().port;
        resolve();
      });
    });
  });

  after(async () => {
    if (engine) {
      await engine.close();
    }
    if (workerServer) {
      await new Promise((resolve) => workerServer.close(resolve));
    }
    if (fixtureServer) {
      await new Promise((resolve) => fixtureServer.close(resolve));
    }
  });

  async function postRender(payload) {
    return new Promise((resolve, reject) => {
      const data = JSON.stringify(payload);
      const req = http.request(
        {
          hostname: '127.0.0.1',
          port: workerPort,
          path: '/render',
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(data),
          },
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
      req.write(data);
      req.end();
    });
  }

  test('GET /ready returns 200 and confirms Chromium readiness', async () => {
    const res = await new Promise((resolve, reject) => {
      http.get(`http://127.0.0.1:${workerPort}/ready`, (r) => {
        let chunks = [];
        r.on('data', (c) => chunks.push(c));
        r.on('end', () => {
          resolve({
            status: r.statusCode,
            json: JSON.parse(Buffer.concat(chunks).toString('utf8')),
          });
        });
      }).on('error', reject);
    });

    assert.strictEqual(res.status, 200);
    assert.strictEqual(res.json.status, 'ready');
    assert.strictEqual(res.json.browser_ready, true);
  });

  test('renders dynamic JS SPA page with selector wait and extracts rendered DOM', async () => {
    const res = await postRender({
      url: `http://127.0.0.1:${fixturePort}/dynamic`,
      wait_until: 'load',
      wait_for_selector: '#rendered-title',
      timeout_ms: 15000,
      block_subresource_ips: false,
    });

    assert.strictEqual(res.status, 200);
    assert.strictEqual(res.json.status_code, 200);
    assert.strictEqual(res.json.title, 'Dynamic SPA Test');
    assert.ok(res.json.html.includes('Hydrated Content Ready'));
    assert.ok(res.json.html.includes('Extracted via Playwright'));
    assert.ok(res.json.load_time_ms >= 50);

    // Verify console message capture
    const spaLog = res.json.console_messages.find((m) => m.text.includes('[SPA] Hydration finished successfully'));
    assert.ok(spaLog, 'Expected SPA console log to be captured');
  });

  test('follows HTTP redirects and captures redirect provenance chain', async () => {
    const res = await postRender({
      url: `http://127.0.0.1:${fixturePort}/redirect-source`,
      wait_until: 'load',
      timeout_ms: 15000,
      block_subresource_ips: false,
    });

    assert.strictEqual(res.status, 200);
    assert.strictEqual(res.json.status_code, 200);
    assert.strictEqual(res.json.title, 'Destination Page');
    assert.ok(res.json.html.includes('Arrived'));
    assert.ok(res.json.redirects.length >= 1);
    assert.strictEqual(res.json.redirects[0].status_code, 302);
  });

  test('returns 504 SelectorTimeout when wait_for_selector does not appear', async () => {
    const res = await postRender({
      url: `http://127.0.0.1:${fixturePort}/dynamic`,
      wait_until: 'load',
      wait_for_selector: '#element-that-never-exists',
      timeout_ms: 1000, // Short timeout for test speed
      block_subresource_ips: false,
    });

    assert.strictEqual(res.status, 504);
    assert.strictEqual(res.json.error_code, 'SelectorTimeout');
    assert.strictEqual(res.json.status, 504);
  });
});
