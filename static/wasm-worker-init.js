// WASM Worker Pool Initialization
// This script manages the worker pool for multithreaded WASM execution

(function() {
  'use strict';

  // Check if SharedArrayBuffer is available
  if (typeof SharedArrayBuffer === 'undefined') {
    console.warn('[WASM Workers] SharedArrayBuffer not available - threading disabled');
    return;
  }

  // Configuration
  const config = {
    maxWorkers: navigator.hardwareConcurrency || 4,
    minWorkers: 2,
    workerTimeout: 30000, // 30 seconds
    enableLogging: true
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

      } catch (error) {
        this.error('Failed to initialize worker pool', error);
        throw error;
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
            created: Date.now()
          };

          // Set up message handler
          worker.onmessage = (event) => {
            this.handleWorkerMessage(workerId, event.data);
          };

          // Set up error handler
          worker.onerror = (error) => {
            this.error(`Worker ${workerId} error`, error);
            this.handleWorkerError(workerId, error);
          };

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
          this.processPendingTasks();
          break;

        case 'error':
          this.error(`Worker ${workerId} reported error`, message.error);
          break;

        default:
          this.log(`Worker ${workerId} message`, message);
      }
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

    // Execute a task on an available worker
    async executeTask(taskData) {
      if (!this.initialized) {
        throw new Error('Worker pool not initialized');
      }

      return new Promise((resolve, reject) => {
        const task = {
          data: taskData,
          resolve,
          reject,
          timestamp: Date.now()
        };

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

          // Set up one-time message handler for task result
          const onResult = (event) => {
            if (event.data.type === 'task-complete') {
              workerInfo.worker.removeEventListener('message', onResult);
              task.resolve(event.data.result);
            } else if (event.data.type === 'error') {
              workerInfo.worker.removeEventListener('message', onResult);
              task.reject(new Error(event.data.error));
            }
          };

          workerInfo.worker.addEventListener('message', onResult);
          workerInfo.worker.postMessage({
            type: 'task',
            data: task.data
          });

          this.log(`Assigned task to worker ${workerId}`);
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

    // Terminate all workers
    terminate() {
      this.log('Terminating worker pool');

      this.workers.forEach((workerInfo, index) => {
        if (workerInfo && workerInfo.worker) {
          workerInfo.worker.terminate();
          this.log(`Worker ${index} terminated`);
        }
      });

      this.workers = [];
      this.availableWorkers = [];
      this.pendingTasks = [];
      this.initialized = false;
    }

    // Get pool statistics
    getStats() {
      return {
        totalWorkers: this.workers.length,
        availableWorkers: this.availableWorkers.length,
        busyWorkers: this.workers.filter(w => w && w.busy).length,
        pendingTasks: this.pendingTasks.length,
        totalTasks: this.workers.reduce((sum, w) => sum + (w ? w.tasks : 0), 0),
        initialized: this.initialized
      };
    }
  }

  // Export to global scope
  window.WasmWorkerPool = WasmWorkerPool;

  // Auto-initialize if WASM module is detected
  if (window.wasmThreadPool) {
    console.log('[WASM Workers] Auto-initialization enabled');
  }

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
        if (config.enableLogging) {
          console.log('[WASM Workers] Health Check:', {
            totalWorkers: stats.totalWorkers,
            availableWorkers: stats.availableWorkers,
            busyWorkers: stats.busyWorkers,
            pendingTasks: stats.pendingTasks,
            totalTasks: stats.totalTasks
          });
        }

        // Warn if all workers are busy for extended period
        if (stats.busyWorkers === stats.totalWorkers && stats.pendingTasks > 0) {
          console.warn('[WASM Workers] All workers busy with pending tasks - consider increasing pool size');
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

})();
