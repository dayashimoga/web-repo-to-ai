/**
 * @file limiter.js
 * @description Concurrency limiter and bounded FIFO queue for browser render requests.
 */

import { RenderEngineError } from './engine.js';

export const DEFAULT_MAX_CONCURRENT_PAGES = 4;
export const DEFAULT_MAX_QUEUE_SIZE = 20;
export const DEFAULT_QUEUE_TIMEOUT_MS = 15000;

/**
 * Manages concurrency bounding and queued execution of browser sessions.
 */
export class ConcurrencyLimiter {
  /**
   * @param {object} [options={}]
   * @param {number} [options.maxConcurrent] - Maximum simultaneous active renders.
   * @param {number} [options.maxQueueSize] - Maximum number of requests waiting in queue.
   * @param {number} [options.queueTimeoutMs] - Maximum time a request may wait in queue.
   */
  constructor(options = {}) {
    this.maxConcurrent =
      typeof options.maxConcurrent === 'number'
        ? options.maxConcurrent
        : Number.parseInt(process.env.MAX_CONCURRENT_PAGES || process.env.CONCURRENCY_LIMIT || '', 10) ||
          DEFAULT_MAX_CONCURRENT_PAGES;

    this.maxQueueSize =
      typeof options.maxQueueSize === 'number'
        ? options.maxQueueSize
        : Number.parseInt(process.env.MAX_QUEUE_SIZE || '', 10) ||
          DEFAULT_MAX_QUEUE_SIZE;

    this.queueTimeoutMs =
      typeof options.queueTimeoutMs === 'number'
        ? options.queueTimeoutMs
        : Number.parseInt(process.env.QUEUE_TIMEOUT_MS || '', 10) ||
          DEFAULT_QUEUE_TIMEOUT_MS;

    this.activeCount = 0;
    this.queue = [];
  }

  /**
   * Acquires a concurrency token or waits in the bounded queue.
   *
   * @returns {Promise<void>}
   * @throws {RenderEngineError} If queue capacity is exceeded (429) or queue wait times out (504).
   */
  async acquire() {
    if (this.activeCount < this.maxConcurrent) {
      this.activeCount++;
      return;
    }

    if (this.queue.length >= this.maxQueueSize) {
      throw new RenderEngineError(
        'ConcurrencyLimitExceeded',
        `Worker concurrency limit reached (${this.maxConcurrent} active renders, queue capacity of ${this.maxQueueSize} full). Try again later.`,
        429
      );
    }

    return new Promise((resolve, reject) => {
      let timer = null;
      const queueItem = {
        resolve: () => {
          if (timer) clearTimeout(timer);
          resolve();
        },
        reject: (err) => {
          if (timer) clearTimeout(timer);
          reject(err);
        },
      };

      timer = setTimeout(() => {
        const index = this.queue.indexOf(queueItem);
        if (index !== -1) {
          this.queue.splice(index, 1);
          queueItem.reject(
            new RenderEngineError(
              'QueueTimeout',
              `Timed out after ${this.queueTimeoutMs}ms waiting in browser worker queue.`,
              504
            )
          );
        }
      }, this.queueTimeoutMs);

      this.queue.push(queueItem);
    });
  }

  /**
   * Releases a concurrency token, allowing the next queued request to proceed.
   */
  release() {
    if (this.queue.length > 0) {
      const next = this.queue.shift();
      next.resolve();
    } else {
      this.activeCount = Math.max(0, this.activeCount - 1);
    }
  }

  /**
   * Returns current concurrency and queue statistics.
   *
   * @returns {{ active: number, queued: number, max_concurrent: number, max_queue_size: number }}
   */
  getStats() {
    return {
      active: this.activeCount,
      queued: this.queue.length,
      max_concurrent: this.maxConcurrent,
      max_queue_size: this.maxQueueSize,
    };
  }
}
