# Frontend Flow Diagram

## Initialization Sequence

```
┌─────────────────────────────────────────────────────────────────────┐
│                          USER LOADS PAGE                             │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    index.html Loads (0ms)                            │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ • Display loading overlay                                      │ │
│  │ • Show animated spinner                                        │ │
│  │ • Initialize progress bar (0%)                                 │ │
│  └────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│              wasm-worker-init.js Loads (~50ms)                       │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ • Define WasmWorkerPool class                                  │ │
│  │ • Define WorkerHealthMonitor class                             │ │
│  │ • Export to window.WasmWorkerPool                              │ │
│  └────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│             rusty-audio-init.js Executes (~100ms)                    │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                  INITIALIZATION PIPELINE                        │ │
│  ├────────────────────────────────────────────────────────────────┤ │
│  │ 1. Feature Detection (5% progress)                             │ │
│  │    ├─ Check WebAssembly                     ✓ CRITICAL         │ │
│  │    ├─ Check SharedArrayBuffer               ○ OPTIONAL         │ │
│  │    ├─ Check Atomics                         ○ OPTIONAL         │ │
│  │    ├─ Check crossOriginIsolated             ○ OPTIONAL         │ │
│  │    ├─ Check Service Worker                  ○ OPTIONAL         │ │
│  │    ├─ Check Web Audio API                   ○ OPTIONAL         │ │
│  │    ├─ Check WebGPU                          ○ OPTIONAL         │ │
│  │    └─ Detect Hardware Concurrency           ○ INFO             │ │
│  │                                                                 │ │
│  │ 2. Display Features (10% progress)                             │ │
│  │    ├─ Render feature grid                                      │ │
│  │    ├─ Show ✓/✗/⚠ indicators                                    │ │
│  │    └─ Log to console                                           │ │
│  │                                                                 │ │
│  │ 3. Register Service Worker (15% progress)                      │ │
│  │    ├─ navigator.serviceWorker.register()                       │ │
│  │    ├─ Setup update listeners                                   │ │
│  │    └─ Continue if fails (non-blocking)                         │ │
│  │                                                                 │ │
│  │ 4. Initialize Worker Pool (20% progress)                       │ │
│  │    ├─ Check SharedArrayBuffer available                        │ │
│  │    ├─ Create WasmWorkerPool instance                           │ │
│  │    ├─ Set maxWorkers = navigator.hardwareConcurrency          │ │
│  │    ├─ Set minWorkers = min(2, maxWorkers)                     │ │
│  │    └─ Display thread visualization grid                        │ │
│  │                                                                 │ │
│  │ 5. Hook Fetch for Progress (25% progress)                      │ │
│  │    ├─ Override window.fetch                                    │ │
│  │    ├─ Track WASM download via ReadableStream                  │ │
│  │    └─ Update progress bar in real-time                         │ │
│  └────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│               WASM Module Download (25-75% progress)                 │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ Trunk auto-injects script tag for rusty-audio.js              │ │
│  │         ↓                                                       │ │
│  │ rusty-audio.js loads (~100KB)                                  │ │
│  │         ↓                                                       │ │
│  │ Fetches rusty-audio_bg.wasm (~5.9MB)                           │ │
│  │         ↓                                                       │ │
│  │ Progress tracked via hooked fetch()                            │ │
│  │   ├─ Update progress bar: 25% → 75%                            │ │
│  │   ├─ Display MB downloaded / total MB                          │ │
│  │   └─ Status: "Downloading WASM module..."                      │ │
│  └────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│              WASM Initialization (75-95% progress)                   │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ 1. Wait for wasm_bindgen availability (75%)                    │ │
│  │    ├─ Poll for typeof wasm_bindgen !== 'undefined'             │ │
│  │    ├─ Max wait: 5 seconds                                      │ │
│  │    └─ Throw error if timeout                                   │ │
│  │                                                                 │ │
│  │ 2. Compile WASM Module (80%)                                   │ │
│  │    ├─ await wasm_bindgen()                                     │ │
│  │    ├─ Compile to native code                                   │ │
│  │    └─ Status: "Compiling WASM module..."                       │ │
│  │                                                                 │ │
│  │ 3. Initialize Shared Memory (85%)                              │ │
│  │    ├─ Access wasm_bindgen.memory                               │ │
│  │    ├─ Verify SharedArrayBuffer backing                         │ │
│  │    └─ Prepare for worker sharing                               │ │
│  │                                                                 │ │
│  │ 4. Initialize Worker Pool with WASM (90-95%)                   │ │
│  │    ├─ await workerPool.init(module, memory, workerScript)     │ │
│  │    ├─ Create min workers (2-4)                                 │ │
│  │    ├─ Send init message to each worker                         │ │
│  │    ├─ Wait for init-complete from workers                      │ │
│  │    └─ Update thread indicators to "idle"                       │ │
│  └────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  Finalization (95-100% progress)                     │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ 1. Setup Error Handlers                                        │ │
│  │    ├─ canvas.addEventListener('webglcontextlost')              │ │
│  │    ├─ canvas.addEventListener('webglcontextrestored')          │ │
│  │    ├─ window.addEventListener('error')                         │ │
│  │    └─ window.addEventListener('unhandledrejection')            │ │
│  │                                                                 │ │
│  │ 2. Setup Keyboard Shortcuts                                    │ │
│  │    ├─ Ctrl+Shift+P → Toggle perf monitor                       │ │
│  │    └─ Ctrl+Shift+R → Force reload                              │ │
│  │                                                                 │ │
│  │ 3. Start Performance Monitoring                                │ │
│  │    ├─ requestAnimationFrame loop                               │ │
│  │    ├─ Update FPS counter every second                          │ │
│  │    ├─ Update frame time every frame                            │ │
│  │    ├─ Update memory usage (if available)                       │ │
│  │    └─ Show perf monitor after 2 seconds                        │ │
│  │                                                                 │ │
│  │ 4. Hide Loading Overlay (100%)                                 │ │
│  │    ├─ updateMessage('Launching Rusty Audio...')                │ │
│  │    ├─ setTimeout(() => hideLoading(), 500)                     │ │
│  │    └─ Fade out with CSS transition                             │ │
│  └────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    APPLICATION RUNNING                               │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ • egui/eframe renders to canvas                                │ │
│  │ • Audio processing in worker threads (if available)            │ │
│  │ • Performance monitor displays real-time metrics               │ │
│  │ • Service worker caches assets for offline use                 │ │
│  │ • Worker health monitor tracks pool status                     │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Interaction Diagram

```
┌──────────────────┐         ┌──────────────────┐         ┌──────────────────┐
│                  │         │                  │         │                  │
│   index.html     │◄────────┤ rusty-audio-init │────────►│  WasmWorkerPool  │
│   (UI Layer)     │         │   (Controller)   │         │   (Threading)    │
│                  │         │                  │         │                  │
└────────┬─────────┘         └────────┬─────────┘         └────────┬─────────┘
         │                            │                            │
         │ Displays                   │ Orchestrates               │ Manages
         │                            │                            │
         ▼                            ▼                            ▼
┌──────────────────┐         ┌──────────────────┐         ┌──────────────────┐
│                  │         │                  │         │                  │
│ Loading Overlay  │         │ Service Worker   │         │  Worker Threads  │
│ Progress Bar     │         │  (Caching)       │         │   (0...N-1)      │
│ Feature Grid     │         │                  │         │                  │
│ Thread Viz       │         └──────────────────┘         └──────────────────┘
│ Perf Monitor     │                  │
│                  │                  │ Caches
└──────────────────┘                  ▼
                            ┌──────────────────┐
                            │                  │
                            │  Static Assets   │
                            │  • HTML          │
                            │  • WASM          │
                            │  • JS            │
                            │  • Icons         │
                            │                  │
                            └──────────────────┘
```

## Data Flow - WASM Module Initialization

```
┌─────────────┐
│   Browser   │
└──────┬──────┘
       │
       │ 1. Load index.html
       ▼
┌─────────────────────────────────────────┐
│          index.html                      │
│  ┌────────────────────────────────────┐ │
│  │ <script src="wasm-worker-init.js"> │ │──┐
│  │ <script src="rusty-audio-init.js"> │ │  │
│  │ <script> Trunk injects here </script>│  │
│  └────────────────────────────────────┘ │  │
└─────────────────────────────────────────┘  │
                                              │
       ┌──────────────────────────────────────┘
       │
       │ 2. Trunk-injected script loads WASM
       ▼
┌──────────────────────────────────────────────────────────┐
│              rusty-audio.js                               │
│  (Generated by wasm-bindgen)                             │
│  ┌────────────────────────────────────────────────────┐  │
│  │ export async function wasm_bindgen(input) {        │  │
│  │   const imports = getImports();                    │  │
│  │   const { instance, module } = await load(input); │  │
│  │   return finalizeInit(instance, module);          │  │
│  │ }                                                   │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────┬───────────────────────────────────────┘
                   │
                   │ 3. Fetch WASM binary
                   ▼
┌──────────────────────────────────────────────────────────┐
│       Hooked fetch() in rusty-audio-init.js              │
│  ┌────────────────────────────────────────────────────┐  │
│  │ Track download progress:                           │  │
│  │   receivedLength += chunk.length                   │  │
│  │   progress = receivedLength / contentLength        │  │
│  │   updateProgress(25 + progress * 50)               │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────┬───────────────────────────────────────┘
                   │
                   │ 4. WASM downloaded (5.9MB)
                   ▼
┌──────────────────────────────────────────────────────────┐
│          WebAssembly.compile()                            │
│  Compile WASM bytecode to native machine code            │
│  (Takes 500ms - 2s depending on device)                  │
└──────────────────┬───────────────────────────────────────┘
                   │
                   │ 5. Module compiled
                   ▼
┌──────────────────────────────────────────────────────────┐
│       WebAssembly.instantiate(module, imports)            │
│  ┌────────────────────────────────────────────────────┐  │
│  │ Create instance with:                              │  │
│  │   • Linear memory (shared if threading)            │  │
│  │   • Tables for indirect calls                      │  │
│  │   • Imported JS functions                          │  │
│  │   • Exported WASM functions                        │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────┬───────────────────────────────────────┘
                   │
                   │ 6. Instance ready
                   ▼
┌──────────────────────────────────────────────────────────┐
│         Initialize Worker Pool (if SAB available)         │
│  ┌────────────────────────────────────────────────────┐  │
│  │ for (i = 0; i < minWorkers; i++) {                 │  │
│  │   worker = new Worker('rusty-audio.worker.js')     │  │
│  │   worker.postMessage({                             │  │
│  │     type: 'init',                                  │  │
│  │     module: wasmModule,                            │  │
│  │     memory: sharedMemory                           │  │
│  │   })                                               │  │
│  │   await worker.onmessage('init-complete')          │  │
│  │ }                                                   │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────┬───────────────────────────────────────┘
                   │
                   │ 7. All workers initialized
                   ▼
┌──────────────────────────────────────────────────────────┐
│                 APPLICATION READY                         │
│  • Hide loading overlay (fade out)                       │
│  • Start performance monitoring                          │
│  • Enable user interaction                               │
│  • Begin rendering loop                                  │
└──────────────────────────────────────────────────────────┘
```

## Worker Pool Task Distribution

```
┌────────────────────────────────────────────────────────────────────┐
│                        MAIN THREAD                                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  WasmWorkerPool                                              │  │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │  │
│  │  │ Task Queue     │  │ Worker Array   │  │ Available      │ │  │
│  │  │ [Task1, Task2] │  │ [W0, W1, W2]   │  │ [0, 2]         │ │  │
│  │  └────────┬───────┘  └────────┬───────┘  └────────┬───────┘ │  │
│  │           │                   │                   │          │  │
│  └───────────┼───────────────────┼───────────────────┼──────────┘  │
│              │                   │                   │             │
│              │ 1. Push task      │                   │             │
│              ▼                   │                   │             │
│        processPendingTasks()     │                   │             │
│              │                   │                   │             │
│              │ 2. Get available  │                   │             │
│              │    worker         │                   │             │
│              ├───────────────────┴───────────────────┤             │
│              │                                       │             │
│              │ 3. Pop worker 0                       │             │
│              │    from available                     │             │
│              ▼                                       ▼             │
│        Assign Task1 to Worker 0                  Available = [2]  │
│              │                                                     │
└──────────────┼─────────────────────────────────────────────────────┘
               │
               │ 4. postMessage({ type: 'task', data: Task1 })
               ▼
┌────────────────────────────────────────────────────────────────────┐
│                         WORKER 0                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ onmessage(event) {                                           │  │
│  │   if (event.data.type === 'task') {                          │  │
│  │     result = processTask(event.data.data)                    │  │
│  │     postMessage({ type: 'task-complete', result })           │  │
│  │   }                                                           │  │
│  │ }                                                             │  │
│  └──────────────────────────────────────────────────────────────┘  │
└──────────────────┬─────────────────────────────────────────────────┘
                   │
                   │ 5. postMessage({ type: 'task-complete', result })
                   ▼
┌────────────────────────────────────────────────────────────────────┐
│                        MAIN THREAD                                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ handleWorkerMessage(0, message) {                            │  │
│  │   worker[0].busy = false                                     │  │
│  │   worker[0].tasks++                                          │  │
│  │   availableWorkers.push(0)                                   │  │
│  │   processPendingTasks()  // Check for more tasks             │  │
│  │ }                                                             │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  Available = [2, 0]  ← Worker 0 now available again                │
└─────────────────────────────────────────────────────────────────────┘
```

## Performance Monitor Update Loop

```
┌──────────────────────────────────────────────────────────────┐
│              requestAnimationFrame Loop                       │
└──────────────────┬───────────────────────────────────────────┘
                   │
                   │ Every frame (~16.7ms @ 60Hz)
                   ▼
┌──────────────────────────────────────────────────────────────┐
│          updatePerformanceStats()                             │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ 1. Calculate Frame Time                                │  │
│  │    deltaTime = now - lastFrameTime                     │  │
│  │    ├─ Update DOM: #perf-frame-time                     │  │
│  │    └─ Color code: green/yellow/red                     │  │
│  │                                                         │  │
│  │ 2. Update FPS (every 1000ms)                           │  │
│  │    frameCount++                                        │  │
│  │    if (elapsed >= 1000) {                              │  │
│  │      fps = frameCount                                  │  │
│  │      ├─ Update DOM: #perf-fps                          │  │
│  │      └─ Color code based on fps                        │  │
│  │    }                                                    │  │
│  │                                                         │  │
│  │ 3. Update Memory (if available)                        │  │
│  │    memoryMB = performance.memory.usedJSHeapSize / 1MB  │  │
│  │    ├─ Update DOM: #perf-memory                         │  │
│  │    └─ Warn if > 500MB                                  │  │
│  │                                                         │  │
│  │ 4. Update Thread Count                                 │  │
│  │    if (workerPool) {                                   │  │
│  │      stats = workerPool.getStats()                     │  │
│  │      ├─ Update DOM: #perf-threads                      │  │
│  │      └─ Update thread indicators (visual grid)         │  │
│  │    }                                                    │  │
│  │                                                         │  │
│  │ 5. Audio Latency (set by Rust backend)                │  │
│  │    // Updated via window.rustyAudio.setAudioLatency()  │  │
│  │    ├─ Update DOM: #perf-audio-latency                  │  │
│  │    └─ Color code: green <25ms, yellow <50ms, red >50ms│  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────┬───────────────────────────────────────────┘
                   │
                   │ requestAnimationFrame(updatePerformanceStats)
                   │
                   └─────────────┐
                                 │
                   Loop continues ▼
```

## Error Handling Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Error Scenarios                           │
└──────────────────┬──────────────────────────────────────────┘
                   │
        ┌──────────┼──────────┬──────────┬──────────┐
        │          │          │          │          │
        ▼          ▼          ▼          ▼          ▼
   ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
   │No WASM │ │No SAB  │ │Worker  │ │WGPU    │ │Network │
   │        │ │        │ │Error   │ │Error   │ │Error   │
   └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘
       │          │          │          │          │
       │          │          │          │          │
       ▼          ▼          ▼          ▼          ▼
┌──────────────────────────────────────────────────────────────┐
│                  Error Handler Routing                        │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                                                         │  │
│  │ CRITICAL (halt initialization):                        │  │
│  │   • No WebAssembly support                             │  │
│  │   • WASM compile/load failure                          │  │
│  │   └─► showError() + display overlay                    │  │
│  │                                                         │  │
│  │ WARNING (continue degraded):                           │  │
│  │   • No SharedArrayBuffer (single-threaded mode)        │  │
│  │   • No WebGPU (fallback to WebGL)                      │  │
│  │   └─► console.warn() + update UI status                │  │
│  │                                                         │  │
│  │ NON-BLOCKING (log only):                               │  │
│  │   • Service Worker registration fails                  │  │
│  │   • Worker health check timeout                        │  │
│  │   └─► console.log() + continue                         │  │
│  │                                                         │  │
│  │ RECOVERABLE (retry):                                   │  │
│  │   • Worker initialization timeout                      │  │
│  │   • WGPU context loss                                  │  │
│  │   └─► Attempt recovery + log                           │  │
│  │                                                         │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Service Worker Cache Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                      Browser Request                         │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   │ Intercepted by Service Worker
                   ▼
┌─────────────────────────────────────────────────────────────┐
│              Service Worker Fetch Handler                    │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ 1. Check request type                                 │  │
│  │    ├─ HTML (navigation)?     → Network-first          │  │
│  │    ├─ WASM/JS?               → Cache-first            │  │
│  │    └─ Static asset?          → Cache-first            │  │
│  └───────────────────────────────────────────────────────┘  │
└──────────────────┬──────────────────────────────────────────┘
                   │
        ┌──────────┴──────────┐
        │                     │
        ▼                     ▼
┌──────────────────┐   ┌──────────────────┐
│  Network-First   │   │   Cache-First    │
│  (HTML)          │   │   (WASM/JS)      │
└────┬─────────────┘   └────┬─────────────┘
     │                      │
     │ 1. Try network       │ 1. Check cache
     ▼                      ▼
┌─────────────┐        ┌─────────────┐
│  Network    │        │   Cache     │
└─────┬───┬───┘        └─────┬───┬───┘
      │   │ Fail            │   │ Miss
      │   └──────┐          │   │
      ▼          ▼          ▼   ▼
┌──────────┐ ┌─────────┐ ┌──────────┐
│ Success  │ │  Cache  │ │ Network  │
│          │ │         │ │          │
│ ├─Cache  │ │         │ │          │
│ └─Return │ │         │ │          │
└──────────┘ └─────────┘ └────┬─────┘
                              │
                              ▼
                         ┌─────────┐
                         │ Success │
                         │         │
                         │ ├─Cache │
                         │ └─Return│
                         └─────────┘
```

This comprehensive flow diagram shows the complete initialization sequence, component interactions, data flows, error handling, and caching strategies for the enhanced multithreaded WASM frontend.
