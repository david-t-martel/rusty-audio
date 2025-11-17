/**
 * Custom Playwright Reporter for Performance Metrics
 *
 * Collects and reports performance data from WASM tests:
 * - FPS measurements
 * - Memory usage
 * - Load times
 * - Worker pool statistics
 */

import {
  Reporter,
  FullConfig,
  Suite,
  TestCase,
  TestResult,
  FullResult,
} from '@playwright/test/reporter';
import * as fs from 'fs';
import * as path from 'path';

interface PerformanceData {
  testName: string;
  browser: string;
  metrics: {
    fps?: number;
    frameTime?: number;
    memory?: number;
    audioLatency?: number;
    loadTime?: number;
  };
  timestamp: string;
  passed: boolean;
  duration: number;
}

class PerformanceReporter implements Reporter {
  private performanceData: PerformanceData[] = [];
  private outputPath: string;

  constructor(options: { outputFile?: string } = {}) {
    this.outputPath = options.outputFile || 'performance-data/performance-metrics.json';
  }

  onBegin(config: FullConfig, suite: Suite) {
    console.log(`\n📊 Performance Reporter initialized`);
    console.log(`   Output: ${this.outputPath}\n`);
  }

  onTestEnd(test: TestCase, result: TestResult) {
    // Extract performance metrics from test attachments or metadata
    const performanceMetric = result.attachments.find(
      (a) => a.name === 'performance-metrics'
    );

    if (performanceMetric && performanceMetric.body) {
      try {
        const metrics = JSON.parse(performanceMetric.body.toString());
        const data: PerformanceData = {
          testName: test.title,
          browser: test.parent.project()?.name || 'unknown',
          metrics,
          timestamp: new Date().toISOString(),
          passed: result.status === 'passed',
          duration: result.duration,
        };

        this.performanceData.push(data);
      } catch (error) {
        // Ignore parse errors
      }
    }
  }

  onEnd(result: FullResult) {
    if (this.performanceData.length === 0) {
      console.log('⚠️  No performance data collected');
      return;
    }

    // Ensure output directory exists
    const outputDir = path.dirname(this.outputPath);
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    // Write performance data
    fs.writeFileSync(
      this.outputPath,
      JSON.stringify(this.performanceData, null, 2)
    );

    // Generate performance summary
    this.generatePerformanceSummary();

    console.log(`\n✅ Performance data saved: ${this.outputPath}`);
  }

  private generatePerformanceSummary() {
    const summary: any = {
      totalTests: this.performanceData.length,
      browsers: {},
      timestamp: new Date().toISOString(),
    };

    // Group by browser
    this.performanceData.forEach((data) => {
      if (!summary.browsers[data.browser]) {
        summary.browsers[data.browser] = {
          tests: 0,
          avgFPS: 0,
          avgMemory: 0,
          avgLoadTime: 0,
          metrics: [],
        };
      }

      const browserSummary = summary.browsers[data.browser];
      browserSummary.tests++;
      browserSummary.metrics.push(data.metrics);

      // Calculate averages
      if (data.metrics.fps) {
        browserSummary.avgFPS =
          (browserSummary.avgFPS * (browserSummary.tests - 1) + data.metrics.fps) /
          browserSummary.tests;
      }
      if (data.metrics.memory) {
        browserSummary.avgMemory =
          (browserSummary.avgMemory * (browserSummary.tests - 1) + data.metrics.memory) /
          browserSummary.tests;
      }
      if (data.metrics.loadTime) {
        browserSummary.avgLoadTime =
          (browserSummary.avgLoadTime * (browserSummary.tests - 1) + data.metrics.loadTime) /
          browserSummary.tests;
      }
    });

    // Write summary
    const summaryPath = path.join(
      path.dirname(this.outputPath),
      'performance-summary.json'
    );
    fs.writeFileSync(summaryPath, JSON.stringify(summary, null, 2));

    // Print summary to console
    console.log('\n📊 Performance Summary:');
    Object.entries(summary.browsers).forEach(([browser, data]: [string, any]) => {
      console.log(`\n  ${browser}:`);
      console.log(`    Tests: ${data.tests}`);
      if (data.avgFPS > 0) console.log(`    Avg FPS: ${data.avgFPS.toFixed(1)}`);
      if (data.avgMemory > 0) console.log(`    Avg Memory: ${data.avgMemory.toFixed(1)} MB`);
      if (data.avgLoadTime > 0) console.log(`    Avg Load Time: ${data.avgLoadTime.toFixed(0)}ms`);
    });
  }
}

export default PerformanceReporter;
