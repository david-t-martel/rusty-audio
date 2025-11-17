// Test Report Generator for WASM Threading
// Run this in browser console to generate a complete test report

(async function generateTestReport() {
  console.log('🧪 Generating WASM Threading Test Report...\n');

  const report = {
    timestamp: new Date().toISOString(),
    url: window.location.href,
    browser: {},
    features: {},
    headers: {},
    serviceWorker: {},
    performance: {},
    recommendations: []
  };

  // Browser Information
  report.browser = {
    userAgent: navigator.userAgent,
    platform: navigator.platform,
    language: navigator.language,
    hardwareConcurrency: navigator.hardwareConcurrency || 'Unknown',
    deviceMemory: navigator.deviceMemory ? `${navigator.deviceMemory} GB` : 'Unknown',
    connection: navigator.connection?.effectiveType || 'Unknown',
    online: navigator.onLine,
    vendor: navigator.vendor,
    maxTouchPoints: navigator.maxTouchPoints || 0
  };

  // Feature Detection
  report.features = {
    webAssembly: {
      supported: typeof WebAssembly !== 'undefined',
      version: typeof WebAssembly !== 'undefined' ? 'WebAssembly 1.0' : 'N/A'
    },
    sharedArrayBuffer: {
      supported: typeof SharedArrayBuffer !== 'undefined',
      reason: typeof SharedArrayBuffer === 'undefined' ?
        'Cross-origin isolation not enabled or browser does not support SAB' :
        'Available'
    },
    crossOriginIsolated: {
      enabled: window.crossOriginIsolated === true,
      value: window.crossOriginIsolated
    },
    serviceWorker: {
      supported: 'serviceWorker' in navigator,
      api: typeof navigator.serviceWorker
    },
    webAudio: {
      supported: typeof AudioContext !== 'undefined' || typeof webkitAudioContext !== 'undefined',
      constructor: typeof AudioContext !== 'undefined' ? 'AudioContext' :
                   typeof webkitAudioContext !== 'undefined' ? 'webkitAudioContext' : 'None'
    },
    webGL: (() => {
      try {
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
        if (gl) {
          return {
            supported: true,
            renderer: gl.getParameter(gl.RENDERER),
            vendor: gl.getParameter(gl.VENDOR),
            version: gl.getParameter(gl.VERSION),
            shadingLanguageVersion: gl.getParameter(gl.SHADING_LANGUAGE_VERSION)
          };
        }
        return { supported: false };
      } catch (e) {
        return { supported: false, error: e.message };
      }
    })(),
    atomics: {
      supported: typeof Atomics !== 'undefined'
    }
  };

  // HTTP Headers Check
  try {
    const response = await fetch(window.location.href, { method: 'HEAD' });
    report.headers = {
      'cross-origin-opener-policy': response.headers.get('Cross-Origin-Opener-Policy'),
      'cross-origin-embedder-policy': response.headers.get('Cross-Origin-Embedder-Policy'),
      'cross-origin-resource-policy': response.headers.get('Cross-Origin-Resource-Policy'),
      'content-security-policy': response.headers.get('Content-Security-Policy'),
      'cache-control': response.headers.get('Cache-Control'),
      'content-type': response.headers.get('Content-Type')
    };
  } catch (error) {
    report.headers = { error: error.message };
  }

  // Service Worker Check
  if ('serviceWorker' in navigator) {
    try {
      const registration = await navigator.serviceWorker.getRegistration();
      if (registration) {
        report.serviceWorker = {
          registered: true,
          state: registration.active?.state || 'unknown',
          scope: registration.scope,
          updateViaCache: registration.updateViaCache,
          waiting: !!registration.waiting,
          installing: !!registration.installing
        };
      } else {
        report.serviceWorker = { registered: false };
      }
    } catch (error) {
      report.serviceWorker = { error: error.message };
    }
  } else {
    report.serviceWorker = { supported: false };
  }

  // Performance Metrics
  const nav = performance.getEntriesByType('navigation')[0];
  if (nav) {
    report.performance = {
      domContentLoaded: Math.round(nav.domContentLoadedEventEnd),
      loadComplete: Math.round(nav.loadEventEnd),
      domInteractive: Math.round(nav.domInteractive),
      transferSize: nav.transferSize,
      encodedBodySize: nav.encodedBodySize,
      decodedBodySize: nav.decodedBodySize,
      type: nav.type
    };
  }

  // Memory (if available)
  if (performance.memory) {
    report.performance.memory = {
      usedJSHeapSize: Math.round(performance.memory.usedJSHeapSize / 1024 / 1024) + ' MB',
      totalJSHeapSize: Math.round(performance.memory.totalJSHeapSize / 1024 / 1024) + ' MB',
      jsHeapSizeLimit: Math.round(performance.memory.jsHeapSizeLimit / 1024 / 1024) + ' MB'
    };
  }

  // Generate Recommendations
  const recommendations = [];

  // Check threading support
  if (!report.features.sharedArrayBuffer.supported) {
    if (!report.features.crossOriginIsolated.enabled) {
      recommendations.push({
        severity: 'high',
        category: 'threading',
        message: 'SharedArrayBuffer not available - cross-origin isolation not enabled',
        solution: 'Ensure HTTPS is used and COOP/COEP headers are set correctly'
      });
    } else {
      recommendations.push({
        severity: 'medium',
        category: 'threading',
        message: 'SharedArrayBuffer not available despite cross-origin isolation',
        solution: 'Browser may not support SharedArrayBuffer. Update to latest version.'
      });
    }
  }

  // Check headers
  if (report.headers['cross-origin-opener-policy'] !== 'same-origin') {
    recommendations.push({
      severity: 'high',
      category: 'headers',
      message: 'Cross-Origin-Opener-Policy header missing or incorrect',
      solution: 'Set header: Cross-Origin-Opener-Policy: same-origin'
    });
  }

  if (report.headers['cross-origin-embedder-policy'] !== 'require-corp') {
    recommendations.push({
      severity: 'high',
      category: 'headers',
      message: 'Cross-Origin-Embedder-Policy header missing or incorrect',
      solution: 'Set header: Cross-Origin-Embedder-Policy: require-corp'
    });
  }

  // Check Service Worker
  if (report.serviceWorker.supported && !report.serviceWorker.registered) {
    recommendations.push({
      severity: 'medium',
      category: 'service-worker',
      message: 'Service Worker not registered',
      solution: 'Check service-worker.js is accessible and registration code is running'
    });
  }

  // Check WebAssembly
  if (!report.features.webAssembly.supported) {
    recommendations.push({
      severity: 'critical',
      category: 'compatibility',
      message: 'WebAssembly not supported',
      solution: 'Update to a modern browser (Chrome 60+, Firefox 52+, Safari 11+, Edge 79+)'
    });
  }

  // Check WebGL
  if (!report.features.webGL.supported) {
    recommendations.push({
      severity: 'medium',
      category: 'graphics',
      message: 'WebGL not supported',
      solution: 'Enable hardware acceleration or update GPU drivers'
    });
  }

  report.recommendations = recommendations;

  // Determine overall status
  const criticalIssues = recommendations.filter(r => r.severity === 'critical').length;
  const highIssues = recommendations.filter(r => r.severity === 'high').length;

  report.overallStatus = criticalIssues > 0 ? 'FAILED' :
                        highIssues > 0 ? 'PARTIAL' : 'PASSED';

  // Print Report
  console.log('═══════════════════════════════════════════════════════════════');
  console.log('                  WASM THREADING TEST REPORT                   ');
  console.log('═══════════════════════════════════════════════════════════════');
  console.log(`Status: ${report.overallStatus}`);
  console.log(`Timestamp: ${report.timestamp}`);
  console.log(`URL: ${report.url}`);
  console.log('───────────────────────────────────────────────────────────────');

  console.log('\n📱 BROWSER INFORMATION');
  console.table(report.browser);

  console.log('\n✨ FEATURE SUPPORT');
  console.log('WebAssembly:', report.features.webAssembly.supported ? '✅' : '❌');
  console.log('SharedArrayBuffer:', report.features.sharedArrayBuffer.supported ? '✅' : '❌');
  console.log('Cross-Origin Isolated:', report.features.crossOriginIsolated.enabled ? '✅' : '❌');
  console.log('Service Worker:', report.features.serviceWorker.supported ? '✅' : '❌');
  console.log('Web Audio API:', report.features.webAudio.supported ? '✅' : '❌');
  console.log('WebGL:', report.features.webGL.supported ? '✅' : '❌');
  console.log('Atomics:', report.features.atomics.supported ? '✅' : '❌');

  console.log('\n🌐 HTTP HEADERS');
  console.table(report.headers);

  console.log('\n⚙️ SERVICE WORKER');
  console.table(report.serviceWorker);

  console.log('\n⚡ PERFORMANCE METRICS');
  console.table(report.performance);

  if (recommendations.length > 0) {
    console.log('\n⚠️ RECOMMENDATIONS');
    recommendations.forEach((rec, index) => {
      const icon = rec.severity === 'critical' ? '🔴' :
                   rec.severity === 'high' ? '🟠' :
                   rec.severity === 'medium' ? '🟡' : 'ℹ️';
      console.log(`\n${icon} [${rec.severity.toUpperCase()}] ${rec.category}`);
      console.log(`   Issue: ${rec.message}`);
      console.log(`   Solution: ${rec.solution}`);
    });
  } else {
    console.log('\n✅ NO ISSUES FOUND - All checks passed!');
  }

  console.log('\n═══════════════════════════════════════════════════════════════');
  console.log('📋 Full report saved to window.testReport');
  console.log('📥 Download JSON: copy(JSON.stringify(window.testReport, null, 2))');
  console.log('═══════════════════════════════════════════════════════════════\n');

  // Save to window object
  window.testReport = report;

  // Return report
  return report;
})();

// Helper function to download report
function downloadTestReport() {
  if (!window.testReport) {
    console.error('No test report available. Run generateTestReport() first.');
    return;
  }

  const dataStr = JSON.stringify(window.testReport, null, 2);
  const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);

  const exportFileDefaultName = `wasm-threading-report-${new Date().toISOString().split('T')[0]}.json`;

  const linkElement = document.createElement('a');
  linkElement.setAttribute('href', dataUri);
  linkElement.setAttribute('download', exportFileDefaultName);
  linkElement.click();

  console.log('✅ Report downloaded as', exportFileDefaultName);
}

// Make download function globally available
window.downloadTestReport = downloadTestReport;

console.log('💡 Tip: Run downloadTestReport() to save the report as JSON');
