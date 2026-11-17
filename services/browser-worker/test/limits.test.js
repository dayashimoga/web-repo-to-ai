import { test, describe, before, after } from 'node:test';
import assert from 'node:assert';
import http from 'node:http';
import { ConcurrencyLimiter } from '../src/limiter.js';
import { PlaywrightEngine, RenderEngineError } from '../src/engine.js';
import { createServer } from '../server.js';

describe('ConcurrencyLimiter Unit Tests', () => {
  test('acquires concurrency token up to maxConcurrent', async () => {
    const limiter = new ConcurrencyLimiter({ maxConcurrent: 2, maxQueueSize: 5 });
    await limiter.acquire();
    await limiter.acquire();
    assert.strictEqual(limiter.getStats().active, 2);
    assert.strictEqual(limiter.getStats().queued, 0);

    limiter.release();
    assert.strictEqual(limiter.getStats().active, 1);
    limiter.release();
    assert.strictEqual(limiter.getStats().active, 0);
  });

  test('queues requests when concurrency limit is reached', async () => {
    const limiter = new ConcurrencyLimiter({ maxConcurrent: 1, maxQueueSize: 2 });
    await limiter.acquire();
    assert.strictEqual(limiter.getStats().active, 1);

    let queuedDone = false;
    const queuedPromise = limiter.acquire().then(() => {
      queuedDone = true;
    });

    assert.strictEqual(limiter.getStats().active, 1);
    assert.strictEqual(limiter.getStats().queued, 1);
    assert.strictEqual(queuedDone, false);

    limiter.release();
    await queuedPromise;
    assert.strictEqual(queuedDone, true);
    assert.strictEqual(limiter.getStats().active, 1);
    assert.strictEqual(limiter.getStats().queued, 0);
    limiter.release();
  });

  test('throws 429 ConcurrencyLimitExceeded when queue is full', async () => {
    const limiter = new ConcurrencyLimiter({ maxConcurrent: 1, maxQueueSize: 1 });
    await limiter.acquire();

    // 1 waiter in queue
    const queuedP = limiter.acquire();
    assert.strictEqual(limiter.getStats().queued, 1);

    // Exceed queue capacity
    await assert.rejects(
      async () => {
        await limiter.acquire();
      },
      (err) => {
        assert.ok(err instanceof RenderEngineError);
        assert.strictEqual(err.code, 'ConcurrencyLimitExceeded');
        assert.strictEqual(err.statusCode, 429);
        return true;
      }
    );

    limiter.release();
    await queuedP;
    limiter.release();
  });

  test('throws 504 QueueTimeout when waiting too long in queue', async () => {
    const limiter = new ConcurrencyLimiter({ maxConcurrent: 1, maxQueueSize: 2, queueTimeoutMs: 50 });
    await limiter.acquire();

    await assert.rejects(
      async () => {
        await limiter.acquire();
      },
      (err) => {
        assert.ok(err instanceof RenderEngineError);
        assert.strictEqual(err.code, 'QueueTimeout');
        assert.strictEqual(err.statusCode, 504);
        return true;
      }
    );

    limiter.release();
  });
});

describe('Hardened SSRF & Hostname Security Tests', () => {
  const engine = new PlaywrightEngine();

  test('isPrivateOrLoopbackHost catches advanced private and loopback addresses', () => {
    // IPv4 Loopback range 127.0.0.0/8
    assert.strictEqual(engine.isPrivateOrLoopbackHost('127.0.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('127.1.2.3'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('127.255.255.254'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('localhost'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('api.localhost'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('internal.local'), true);

    // RFC 1918 Private Ranges
    assert.strictEqual(engine.isPrivateOrLoopbackHost('10.0.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('10.254.1.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('172.16.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('172.31.255.254'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('192.168.1.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('192.168.100.50'), true);

    // Carrier-Grade NAT (RFC 6598: 100.64.0.0/10)
    assert.strictEqual(engine.isPrivateOrLoopbackHost('100.64.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('100.127.255.254'), true);

    // Cloud Metadata Services
    assert.strictEqual(engine.isPrivateOrLoopbackHost('169.254.169.254'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('169.254.1.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('metadata.google.internal'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('100.100.100.200'), true);

    // Current network & broadcast
    assert.strictEqual(engine.isPrivateOrLoopbackHost('0.0.0.0'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('255.255.255.255'), true);

    // IPv6 Loopback, ULA, and Link-Local
    assert.strictEqual(engine.isPrivateOrLoopbackHost('::1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('::'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('[::1]'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('fe80::1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('[fe80::1234:5678]'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('fc00::1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('[fd12:3456:789a::1]'), true);

    // IPv4-mapped IPv6
    assert.strictEqual(engine.isPrivateOrLoopbackHost('::ffff:127.0.0.1'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('[::ffff:10.0.0.1]'), true);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('[::ffff:7f00:1]'), true);

    // Public / Safe destinations
    assert.strictEqual(engine.isPrivateOrLoopbackHost('example.com'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('github.com'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('8.8.8.8'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('1.1.1.1'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('172.15.255.255'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('172.32.0.1'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('100.63.255.255'), false);
    assert.strictEqual(engine.isPrivateOrLoopbackHost('100.128.0.1'), false);
  });
});

describe('Live Browser Resource Limits & Sandbox Isolation', () => {
  let fixtureServer;
  let fixturePort;
  let engine;

  before(async () => {
    fixtureServer = http.createServer((req, res) => {
      const url = new URL(req.url, `http://127.0.0.1:${fixturePort}`);

      if (url.pathname === '/dialog-test') {
        res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
        res.end(`<!DOCTYPE html>
<html>
  <head><title>Dialog Test</title></head>
  <body>
    <h1>Dialog Test</h1>
    <script>
      // Prompt modal dialogs which must be automatically dismissed without hanging
      alert('Alert modal');
      confirm('Confirm modal');
      document.body.innerHTML += '<p id="ready">Completed After Dialogs</p>';
    </script>
  </body>
</html>`);
        return;
      }

      if (url.pathname === '/subresource-burst') {
        res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
        res.end(`<!DOCTYPE html>
<html>
  <head><title>Subresource Burst</title></head>
  <body>
    <h1>Burst</h1>
    <img src="/img1.png" />
    <img src="/img2.png" />
    <img src="/img3.png" />
    <img src="/img4.png" />
    <img src="/img5.png" />
    <p id="loaded">Burst Triggered</p>
  </body>
</html>`);
        return;
      }

      if (url.pathname.startsWith('/img')) {
        res.writeHead(200, { 'Content-Type': 'image/png' });
        res.end(Buffer.alloc(10));
        return;
      }

      if (url.pathname === '/huge-page') {
        res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
        // Generates 200 KB of HTML text
        const chunk = '<p>Large content block that repeats.</p>\n'.repeat(5000);
        res.end(`<!DOCTYPE html><html><head><title>Huge</title></head><body>${chunk}</body></html>`);
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

    engine = new PlaywrightEngine({
      maxConcurrent: 2,
      maxQueueSize: 5,
    });
  });

  after(async () => {
    if (engine) {
      await engine.close();
    }
    if (fixtureServer) {
      await new Promise((resolve) => fixtureServer.close(resolve));
    }
  });

  test('auto-dismisses alert and confirm dialogs without freezing navigation', async () => {
    const res = await engine.render({
      url: `http://127.0.0.1:${fixturePort}/dialog-test`,
      wait_until: 'load',
      wait_for_selector: '#ready',
      timeout_ms: 10000,
      viewport: { width: 1280, height: 800 },
      headers: {},
      javascript_enabled: true,
      block_subresource_ips: false,
    });

    assert.strictEqual(res.status_code, 200);
    assert.strictEqual(res.title, 'Dialog Test');
    assert.ok(res.html.includes('Completed After Dialogs'));
  });

  test('enforces max subresource ceiling and captures blocked subresources', async () => {
    const customEngine = new PlaywrightEngine({
      maxSubresources: 2, // Limit to 2 subresources
    });

    try {
      const res = await customEngine.render({
        url: `http://127.0.0.1:${fixturePort}/subresource-burst`,
        wait_until: 'load',
        wait_for_selector: '#loaded',
        timeout_ms: 10000,
        viewport: { width: 1280, height: 800 },
        headers: {},
        javascript_enabled: true,
        block_subresource_ips: false,
      });

      assert.strictEqual(res.status_code, 200);
      assert.ok(res.blocked_subresources.length >= 1);
      const subExceeded = res.blocked_subresources.find((b) => b.reason.includes('Subresource count exceeded'));
      assert.ok(subExceeded, 'Expected at least one subresource to be blocked due to count limit');
    } finally {
      await customEngine.close();
    }
  });

  test('rejects pages exceeding maxHtmlBytes ceiling with 413 PayloadTooLarge', async () => {
    const smallHtmlEngine = new PlaywrightEngine({
      maxHtmlBytes: 1024, // 1 KB ceiling
    });

    try {
      await assert.rejects(
        async () => {
          await smallHtmlEngine.render({
            url: `http://127.0.0.1:${fixturePort}/huge-page`,
            wait_until: 'load',
            timeout_ms: 10000,
            viewport: { width: 1280, height: 800 },
            headers: {},
            javascript_enabled: true,
            block_subresource_ips: false,
          });
        },
        (err) => {
          assert.ok(err instanceof RenderEngineError);
          assert.strictEqual(err.code, 'PayloadTooLarge');
          assert.strictEqual(err.statusCode, 413);
          return true;
        }
      );
    } finally {
      await smallHtmlEngine.close();
    }
  });

  test('HTTP server returns 429 and Retry-After header on concurrency overflow', async () => {
    const singleEngine = new PlaywrightEngine({
      maxConcurrent: 1,
      maxQueueSize: 0, // No queueing allowed -> immediate 429
    });

    const workerServer = createServer({ browserEngine: singleEngine });
    let workerPort;
    await new Promise((resolve) => {
      workerServer.listen(0, '127.0.0.1', () => {
        workerPort = workerServer.address().port;
        resolve();
      });
    });

    try {
      // Launch one render that stays busy
      const p1 = new Promise((resolve) => {
        const req = http.request(
          {
            hostname: '127.0.0.1',
            port: workerPort,
            path: '/render',
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
          },
          (res) => {
            let data = '';
            res.on('data', (c) => (data += c));
            res.on('end', () => resolve(res.statusCode));
          }
        );
        req.write(
          JSON.stringify({
            url: `http://127.0.0.1:${fixturePort}/dialog-test`,
            wait_until: 'load',
            timeout_ms: 5000,
            block_subresource_ips: false,
          })
        );
        req.end();
      });

      // Immediate second request must receive 429
      await new Promise((r) => setTimeout(r, 20)); // Short pause to let p1 acquire
      const res2 = await new Promise((resolve, reject) => {
        const req = http.request(
          {
            hostname: '127.0.0.1',
            port: workerPort,
            path: '/render',
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
          },
          (res) => {
            let data = '';
            res.on('data', (c) => (data += c));
            res.on('end', () => {
              resolve({
                status: res.statusCode,
                headers: res.headers,
                json: JSON.parse(data),
              });
            });
          }
        );
        req.on('error', reject);
        req.write(
          JSON.stringify({
            url: `http://127.0.0.1:${fixturePort}/dialog-test`,
            wait_until: 'load',
            timeout_ms: 5000,
            block_subresource_ips: false,
          })
        );
        req.end();
      });

      assert.strictEqual(res2.status, 429);
      assert.strictEqual(res2.json.error_code, 'ConcurrencyLimitExceeded');
      assert.strictEqual(res2.headers['retry-after'], '5');

      await p1;
    } finally {
      await singleEngine.close();
      await new Promise((resolve) => workerServer.close(resolve));
    }
  });
});
