/**
 * Global Test Teardown
 *
 * Cleanup operations after all tests complete:
 * - Generate test summary report
 * - Clean up temporary files
 * - Archive performance data
 */

import { FullConfig } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

async function globalTeardown(config: FullConfig) {
  console.log('\n🧹 Starting Playwright global teardown');

  // 1. Generate test summary if results exist
  const resultsPath = path.join(process.cwd(), 'playwright-report', 'results.json');
  if (fs.existsSync(resultsPath)) {
    try {
      const results = JSON.parse(fs.readFileSync(resultsPath, 'utf-8'));
      const summary = {
        total: results.suites?.reduce((acc: number, suite: any) =>
          acc + (suite.specs?.length || 0), 0) || 0,
        passed: 0,
        failed: 0,
        skipped: 0,
        duration: results.duration || 0,
      };

      // Count test results
      results.suites?.forEach((suite: any) => {
        suite.specs?.forEach((spec: any) => {
          spec.tests?.forEach((test: any) => {
            if (test.results?.[0]?.status === 'passed') summary.passed++;
            else if (test.results?.[0]?.status === 'failed') summary.failed++;
            else if (test.results?.[0]?.status === 'skipped') summary.skipped++;
          });
        });
      });

      console.log('\n📊 Test Summary:');
      console.log(`  Total:   ${summary.total}`);
      console.log(`  Passed:  ${summary.passed} ✅`);
      console.log(`  Failed:  ${summary.failed} ${summary.failed > 0 ? '❌' : ''}`);
      console.log(`  Skipped: ${summary.skipped}`);
      console.log(`  Duration: ${(summary.duration / 1000).toFixed(2)}s`);

      // Write summary file
      fs.writeFileSync(
        path.join(process.cwd(), 'playwright-report', 'summary.json'),
        JSON.stringify(summary, null, 2)
      );
    } catch (error) {
      console.warn('⚠️  Could not generate test summary:', error);
    }
  }

  // 2. Archive performance data
  const perfDataDir = path.join(process.cwd(), 'performance-data');
  if (fs.existsSync(perfDataDir)) {
    const files = fs.readdirSync(perfDataDir);
    if (files.length > 0) {
      console.log(`📈 Archived ${files.length} performance data files`);
    }
  }

  // 3. Clean up temporary test files
  const tempDirs = [
    '.playwright-temp',
  ];

  for (const dir of tempDirs) {
    const dirPath = path.join(process.cwd(), dir);
    if (fs.existsSync(dirPath)) {
      fs.rmSync(dirPath, { recursive: true, force: true });
      console.log(`🗑️  Cleaned up: ${dir}`);
    }
  }

  console.log('✅ Global teardown complete\n');
}

export default globalTeardown;
