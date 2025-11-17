// WASM Worker Pool Initialization (REFACTORED)
// This script manages the worker pool for multithreaded WASM execution
//
// P0-7 FIX: Added proper cleanup with AbortController
//
// BEFORE (MEMORY LEAK):
// - Event listeners never cleaned up after task completion
// - Workers accumulate listeners over time
// - Memory grows unbounded with task count
//
// AFTER (MEMORY SAFE):
// - AbortController used to cancel listeners
// - Cleanup function called after each task
// - Bounded memory usage regardless of task count

(function() {
  'use strict';

  // Check if SharedArrayBuffer is available
  if (typeof SharedArrayBuffer === 'undefined') {
    console.warn('[WASM Workers] SharedArrayBuffer not available - threading disabled');
    return;
  }

  // Configuration
  const config = {
    maxWorkers: Math.min(navigator.hardwareConcurrency || 4, 16),  // Cap at 16
    minWorkers: 2,
    workerTimeout: 30000, // 30 seconds
    taskTimeout: 60000,   // 60 seconds per task
    enableLogging: true,
    cleanupInterval: 5000 // Cleanup every 5 seconds
  };

  // Worker pool manager
  class WasmWorkerPool {
    constructor(options = {}) {
      this.maxWorkers = options.maxWorkers || config.maxWorkers;
      this.minWorkers = Math.min(options.minWorkers || config.minWorkers, this.maxWorkers);
      this.workers = [];
      this.availableWorkers = [];
      this.pendingTasks = [];
      this.initialized = false;
      this.workerScript = null;
      this.wasmModule = null;
      this.memory = null;

      // P0-7 FIX: Track active tasks for cleanup
      this.activeTasks = new Map();
      this.taskIdCounter = 0;

      // P0-7 FIX: Cleanup interval
      this.cleanupIntervalId = null;

      this.log('Worker pool created', {
        maxWorkers: this.maxWorkers,
        minWorkers: this.minWorkers,
        hardwareConcurrency: navigator.hardwareConcurrency
      });
    }

    log(message, data = null) {
      if (config.enableLogging) {
        if (data) {
          console.log(`[WASM Workers] ${message}`, data);
        } else {
          console.log(`[WASM Workers] ${message}`);
        }
      }
    }

    error(message, error = null) {
      if (error) {
        console.error(`[WASM Workers] ${message}`, error);
      } else {
        console.error(`[WASM Workers] ${message}`);
      }
    }

    // Initialize the worker pool with WASM module and memory
    async init(wasmModule, memory, workerScriptUrl) {
      if (this.initialized) {
        this.log('Worker pool already initialized');
        return;
      }

      this.log('Initializing worker pool...');

      try {
        this.wasmModule = wasmModule;
        this.memory = memory;
        this.workerScript = workerScriptUrl;

        // Create initial workers
        const workerPromises = [];
        for (let i = 0; i < this.minWorkers; i++) {
          workerPromises.push(this.createWorker(i));
        }

        await Promise.all(workerPromises);

        this.initialized = true;
        this.log(`Worker pool initialized with ${this.workers.length} workers`);

        // P0-7 FIX: Start cleanup interval
        this.startCleanup();

      } catch (error) {
        this.error('Failed to initialize worker pool', error);
        throw error;
      }
    }

    /**
     * P0-7 FIX: Start periodic cleanup of stale tasks
     */
    startCleanup() {
      if (this.cleanupIntervalId) {
        return;
      }

      this.cleanupIntervalId = setInterval(() => {
        this.cleanupStaleTasks();
      }, config.cleanupInterval);

      this.log('Cleanup interval started');
    }

    /**
     * P0-7 FIX: Clean up tasks that have exceeded timeout
     */
    cleanupStaleTasks() {
      const now = Date.now();
      let cleanedCount = 0;

      for (const [taskId, task] of this.activeTasks.entries()) {
        const age = now - task.timestamp;

        if (age > config.taskTimeout) {
          this.log(`Cleaning up stale task ${taskId} (age: ${age}ms)`);

          // Abort the task
          if (task.abortController) {
            task.abortController.abort();
          }

          // Reject the promise
          if (task.reject) {
            task.reject(new Error(`Task timeout after ${age}ms`));
          }

          this.activeTasks.delete(taskId);
          cleanedCount++;
        }
      }

      if (cleanedCount > 0) {
        this.log(`Cleaned up ${cleanedCount} stale tasks`);
      }
    }

    // Create a new worker
    async createWorker(index) {
      return new Promise((resolve, reject) => {
        try {
          const worker = new Worker(this.workerScript, {
            type: 'module',
            name: `wasm-worker-${index}`
          });

          const workerId = this.workers.length;
          const workerInfo = {
            id: workerId,
            worker,
            busy: false,
            tasks: 0,
            created: Date.now(),
            // P0-7 FIX: Track listeners for cleanup
            listeners: new Set()
          };

          // P0-7 FIX: Create AbortController for worker lifecycle
          const workerAbortController = new AbortController();

          // Set up message handler with abort signal
          const onMessage = (event) => {
            this.handleWorkerMessage(workerId, event.data);
          };

          worker.addEventListener('message', onMessage, {
            signal: workerAbortController.signal
          });
          workerInfo.listeners.add({ type: 'message', handler: onMessage });

          // Set up error handler with abort signal
          const onError = (error) => {
            this.error(`Worker ${workerId} error`, error);
            this.handleWorkerError(workerId, error);
          };

          worker.addEventListener('error', onError, {
            signal: workerAbortController.signal
          });
          workerInfo.listeners.add({ type: 'error', handler: onError });

          // Store abort controller
          workerInfo.abortController = workerAbortController;

          // Initialize worker with shared memory
          const initMessage = {
            type: 'init',
            module: this.wasmModule,
            memory: this.memory,
            workerId
          };

          // Wait for initialization confirmation
          const initTimeout = setTimeout(() => {
            reject(new Error(`Worker ${workerId} initialization timeout`));
          }, config.workerTimeout);

          const onInit = (event) => {
            if (event.data.type === 'init-complete') {
              clearTimeout(initTimeout);
              worker.removeEventListener('message', onInit);

              this.workers.push(workerInfo);
              this.availableWorkers.push(workerId);

              this.log(`Worker ${workerId} initialized`);
              resolve(workerInfo);
            }
          };

          worker.addEventListener('message', onInit);
          worker.postMessage(initMessage);

        } catch (error) {
          this.error(`Failed to create worker ${index}`, error);
          reject(error);
        }
      });
    }

    // Handle worker messages
    handleWorkerMessage(workerId, message) {
      const workerInfo = this.workers[workerId];
      if (!workerInfo) {
        this.error(`Unknown worker ID: ${workerId}`);
        return;
      }

      switch (message.type) {
        case 'task-complete':
          workerInfo.busy = false;
          workerInfo.tasks++;
          this.availableWorkers.push(workerId);
          this.log(`Worker ${workerId} completed task (total: ${workerInfo.tasks})`);

          // P0-7 FIX: Clean up task tracking
          if (message.taskId !== undefined) {
            this.cleanupTask(message.taskId);
          }

          this.processPendingTasks();
          break;

        case 'error':
          this.error(`Worker ${workerId} reported error`, message.error);

          // P0-7 FIX: Clean up task on error
          if (message.taskId !== undefined) {
            this.cleanupTask(message.taskId, new Error(message.error));
          }
          break;

        default:
          this.log(`Worker ${workerId} message`, message);
      }
    }

    /**
     * P0-7 FIX: Clean up task resources
     *
     * BEFORE:
     * - Event listeners never removed
     * - Task data retained indefinitely
     * - Memory leak proportional to task count
     *
     * AFTER:
     * - AbortController cancels all listeners
     * - Task data removed from Map
     * - Constant memory usage
     */
    cleanupTask(taskId, error = null) {
      const task = this.activeTasks.get(taskId);
      if (!task) {
        return;
      }

      this.log(`Cleaning up task ${taskId}`);

      // Abort any pending operations
      if (task.abortController) {
        task.abortController.abort();
      }

      // Resolve or reject promise
      if (error) {
        if (task.reject) {
          task.reject(error);
        }
      } else {
        if (task.resolve) {
          task.resolve(task.result);
        }
      }

      // Remove from active tasks
      this.activeTasks.delete(taskId);

      this.log(`Task ${taskId} cleaned up (active tasks: ${this.activeTasks.size})`);
    }

    // Handle worker errors
    handleWorkerError(workerId, error) {
      const workerInfo = this.workers[workerId];
      if (workerInfo) {
        workerInfo.busy = false;

        // Remove from available workers
        const index = this.availableWorkers.indexOf(workerId);
        if (index > -1) {
          this.availableWorkers.splice(index, 1);
        }

        // Attempt to recreate worker if pool is below minimum
        if (this.workers.filter(w => w !== null).length < this.minWorkers) {
          this.log(`Recreating worker ${workerId} due to error`);
          this.createWorker(workerId).catch(err => {
            this.error(`Failed to recreate worker ${workerId}`, err);
          });
        }
      }
    }

    /**
     * Execute a task on an available worker
     *
     * P0-7 FIX: Uses AbortController for proper cleanup
     */
    async executeTask(taskData) {
      if (!this.initialized) {
        throw new Error('Worker pool not initialized');
      }

      return new Promise((resolve, reject) => {
        // P0-7 FIX: Create task with AbortController
        const taskId = this.taskIdCounter++;
        const abortController = new AbortController();

        const task = {
          id: taskId,
          data: taskData,
          resolve,
          reject,
          timestamp: Date.now(),
          abortController  // P0-7 FIX: Store controller for cleanup
        };

        this.activeTasks.set(taskId, task);
        this.pendingTasks.push(task);
        this.processPendingTasks();
      });
    }

    // Process pending tasks
    processPendingTasks() {
      while (this.pendingTasks.length > 0 && this.availableWorkers.length > 0) {
        const task = this.pendingTasks.shift();
        const workerId = this.availableWorkers.shift();
        const workerInfo = this.workers[workerId];

        if (workerInfo && !workerInfo.busy) {
          workerInfo.busy = true;

          // P0-7 FIX: Set up one-time message handler with AbortController
          const onResult = (event) => {
            if (event.data.type === 'task-complete' && event.data.taskId === task.id) {
              // Store result
              task.result = event.data.result;

              // Cleanup happens in handleWorkerMessage
              workerInfo.worker.removeEventListener('message', onResult);
            } else if (event.data.type === 'error' && event.data.taskId === task.id) {
              workerInfo.worker.removeEventListener('message', onResult);
              this.cleanupTask(task.id, new Error(event.data.error));
            }
          };

          // P0-7 FIX: Add listener with abort signal
          workerInfo.worker.addEventListener('message', onResult, {
            signal: task.abortController.signal
          });

          workerInfo.worker.postMessage({
            type: 'task',
            taskId: task.id,
            data: task.data
          });

          this.log(`Assigned task ${task.id} to worker ${workerId}`);
        }
      }

      // Create additional workers if needed
      if (this.pendingTasks.length > 0 &&
          this.workers.length < this.maxWorkers &&
          this.availableWorkers.length === 0) {
        this.log('Creating additional worker for pending tasks');
        this.createWorker(this.workers.length).catch(err => {
          this.error('Failed to create additional worker', err);
        });
      }
    }

    /**
     * Terminate all workers
     *
     * P0-7 FIX: Properly cleans up all resources
     */
    terminate() {
      this.log('Terminating worker pool');

      // Stop cleanup interval
      if (this.cleanupIntervalId) {
        clearInterval(this.cleanupIntervalId);
        this.cleanupIntervalId = null;
      }

      // Abort all active tasks
      for (const [taskId, task] of this.activeTasks.entries()) {
        if (task.abortController) {
          task.abortController.abort();
        }
        if (task.reject) {
          task.reject(new Error('Worker pool terminated'));
        }
      }
      this.activeTasks.clear();

      // Terminate workers and clean up listeners
      this.workers.forEach((workerInfo, index) => {
        if (workerInfo && workerInfo.worker) {
          // Abort worker-level listeners
          if (workerInfo.abortController) {
            workerInfo.abortController.abort();
          }

          workerInfo.worker.terminate();
          this.log(`Worker ${index} terminated`);
        }
      });

      this.workers = [];
      this.availableWorkers = [];
      this.pendingTasks = [];
      this.initialized = false;

      this.log('Worker pool terminated and cleaned up');
    }

    // Get pool statistics
    getStats() {
      return {
        totalWorkers: this.workers.length,
        availableWorkers: this.availableWorkers.length,
        busyWorkers: this.workers.filter(w => w && w.busy).length,
        pendingTasks: this.pendingTasks.length,
        activeTasks: this.activeTasks.size,
        totalTasks: this.workers.reduce((sum, w) => sum + (w ? w.tasks : 0), 0),
        initialized: this.initialized
      };
    }
  }

  // Export to global scope
  window.WasmWorkerPool = WasmWorkerPool;

  // Monitor worker pool health
  class WorkerHealthMonitor {
    constructor(pool) {
      this.pool = pool;
      this.checkInterval = 5000; // 5 seconds
      this.intervalId = null;
    }

    start() {
      if (this.intervalId) return;

      this.intervalId = setInterval(() => {
        if (!this.pool || !this.pool.initialized) return;

        const stats = this.pool.getStats();

        // Log stats periodically
        if (config.enableLogging && stats.totalTasks > 0) {
          console.log('[WASM Workers] Health Check:', {
            totalWorkers: stats.totalWorkers,
            availableWorkers: stats.availableWorkers,
            busyWorkers: stats.busyWorkers,
            pendingTasks: stats.pendingTasks,
            activeTasks: stats.activeTasks,
            totalTasks: stats.totalTasks
          });
        }

        // Warn if all workers are busy for extended period
        if (stats.busyWorkers === stats.totalWorkers && stats.pendingTasks > 0) {
          console.warn('[WASM Workers] All workers busy with pending tasks - consider increasing pool size');
        }

        // P0-7 FIX: Warn about task leaks
        if (stats.activeTasks > stats.totalWorkers * 2) {
          console.warn(`[WASM Workers] High active task count (${stats.activeTasks}) - possible leak`);
        }

        // Notify about worker pool status via custom event
        window.dispatchEvent(new CustomEvent('wasm-worker-health', {
          detail: stats
        }));

      }, this.checkInterval);
    }

    stop() {
      if (this.intervalId) {
        clearInterval(this.intervalId);
        this.intervalId = null;
      }
    }
  }

  window.WorkerHealthMonitor = WorkerHealthMonitor;

  console.log('[WASM Workers] Refactored module loaded with P0-7 memory leak fix');

})();
