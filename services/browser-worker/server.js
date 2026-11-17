/**
 * @file server.js
 * @description Main entrypoint for the isolated Playwright browser worker service.
 */

import http from 'node:http';
import { handleRequest, SERVICE_NAME, SERVICE_VERSION } from './src/routes.js';

export const PORT = Number.parseInt(process.env.PORT || '3000', 10);
export const HOST = process.env.HOST || '0.0.0.0';

/**
 * Creates and configures the HTTP server instance.
 *
 * @returns {http.Server}
 */
export function createServer() {
  return http.createServer((req, res) => {
    handleRequest(req, res).catch((err) => {
      console.error('[browser-worker] Unhandled request error:', err);
      if (!res.headersSent) {
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Internal server error', detail: err.message }));
      }
    });
  });
}

// Start server if directly executed
const isDirectExecution = import.meta.url === `file://${process.argv[1].replace(/\\/g, '/')}`;
if (isDirectExecution) {
  const server = createServer();
  server.listen(PORT, HOST, () => {
    console.log(`[${SERVICE_NAME} v${SERVICE_VERSION}] Listening on http://${HOST}:${PORT}`);
  });

  const shutdown = () => {
    console.log(`[${SERVICE_NAME}] Shutting down gracefully...`);
    server.close(() => {
      console.log(`[${SERVICE_NAME}] Server closed.`);
      process.exit(0);
    });
  };

  process.on('SIGINT', shutdown);
  process.on('SIGTERM', shutdown);
}
