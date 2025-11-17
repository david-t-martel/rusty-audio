#!/usr/bin/env node

/**
 * Setup Verification Script
 *
 * Verifies that the development environment is correctly set up:
 * - Required tools installed
 * - Dependencies present
 * - Configuration files valid
 * - File structure correct
 */

import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';
import chalk from 'chalk';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT_DIR = join(__dirname, '..');

let errors = 0;
let warnings = 0;
let passed = 0;

function checkCommand(command, name, required = true) {
  try {
    execSync(`${command} --version`, { stdio: 'ignore' });
    console.log(chalk.green('✓'), name, 'installed');
    passed++;
    return true;
  } catch (error) {
    if (required) {
      console.log(chalk.red('✗'), name, chalk.red('NOT FOUND (required)'));
      errors++;
    } else {
      console.log(chalk.yellow('⚠'), name, chalk.yellow('NOT FOUND (optional)'));
      warnings++;
    }
    return false;
  }
}

function checkFile(path, description, required = true) {
  const fullPath = join(ROOT_DIR, path);
  if (existsSync(fullPath)) {
    console.log(chalk.green('✓'), description);
    passed++;
    return true;
  } else {
    if (required) {
      console.log(chalk.red('✗'), description, chalk.red('MISSING'));
      errors++;
    } else {
      console.log(chalk.yellow('⚠'), description, chalk.yellow('MISSING'));
      warnings++;
    }
    return false;
  }
}

function checkNodeVersion() {
  try {
    const version = execSync('node --version', { encoding: 'utf8' }).trim();
    const majorVersion = parseInt(version.slice(1).split('.')[0]);

    if (majorVersion >= 18) {
      console.log(chalk.green('✓'), `Node.js ${version} (>= 18 required)`);
      passed++;
      return true;
    } else {
      console.log(chalk.red('✗'), `Node.js ${version}`, chalk.red('Version 18+ required'));
      errors++;
      return false;
    }
  } catch (error) {
    console.log(chalk.red('✗'), 'Node.js version check failed');
    errors++;
    return false;
  }
}

function checkRustTarget() {
  try {
    const output = execSync('rustup target list --installed', { encoding: 'utf8' });
    if (output.includes('wasm32-unknown-unknown')) {
      console.log(chalk.green('✓'), 'WASM target installed');
      passed++;
      return true;
    } else {
      console.log(chalk.yellow('⚠'), 'WASM target not installed');
      console.log(chalk.gray('   Run: rustup target add wasm32-unknown-unknown'));
      warnings++;
      return false;
    }
  } catch (error) {
    console.log(chalk.yellow('⚠'), 'Could not check Rust targets');
    warnings++;
    return false;
  }
}

console.log(chalk.bold.cyan('\n=== Development Environment Verification ===\n'));

// Check required tools
console.log(chalk.bold('Required Tools:'));
checkCommand('cargo', 'Rust/Cargo');
checkCommand('rustc', 'Rust Compiler');
checkCommand('trunk', 'Trunk (WASM bundler)');
checkCommand('node', 'Node.js');
checkCommand('npm', 'npm');

// Check Node.js version
checkNodeVersion();

// Check Rust WASM target
checkRustTarget();

// Check optional tools
console.log(chalk.bold('\nOptional Tools:'));
checkCommand('wasm-opt', 'wasm-opt (optimization)', false);
checkCommand('wrangler', 'Wrangler (Cloudflare CLI)', false);
checkCommand('brotli', 'Brotli (compression)', false);
checkCommand('gzip', 'Gzip (compression)', false);

// Check configuration files
console.log(chalk.bold('\nConfiguration Files:'));
checkFile('package.json', 'package.json');
checkFile('wrangler.toml', 'wrangler.toml');
checkFile('Trunk.toml', 'Trunk.toml');
checkFile('Cargo.toml', 'Cargo.toml');

// Check script files
console.log(chalk.bold('\nScript Files:'));
checkFile('scripts/dev-server.js', 'dev-server.js');
checkFile('scripts/build-and-serve.sh', 'build-and-serve.sh');
checkFile('scripts/validate-build.js', 'validate-build.js');
checkFile('scripts/health-check.js', 'health-check.js');
checkFile('scripts/test-threading.js', 'test-threading.js');

// Check documentation
console.log(chalk.bold('\nDocumentation:'));
checkFile('LOCAL_DEV_SETUP.md', 'LOCAL_DEV_SETUP.md');
checkFile('QUICK_START_LOCAL_DEV.md', 'QUICK_START_LOCAL_DEV.md', false);
checkFile('CLAUDE.md', 'CLAUDE.md', false);

// Check source directory
console.log(chalk.bold('\nProject Structure:'));
checkFile('src', 'src/ directory');
checkFile('static', 'static/ directory');
checkFile('index.html', 'index.html');

// Check node_modules
console.log(chalk.bold('\nDependencies:'));
if (existsSync(join(ROOT_DIR, 'node_modules'))) {
  console.log(chalk.green('✓'), 'node_modules/ present');
  passed++;
} else {
  console.log(chalk.yellow('⚠'), 'node_modules/ missing - run: npm install');
  warnings++;
}

// Summary
console.log(chalk.bold('\n=== Verification Summary ===\n'));

const total = passed + warnings + errors;
const percentage = total > 0 ? ((passed / total) * 100).toFixed(1) : 0;

console.log(chalk.green(`✓ Passed: ${passed}`));
if (warnings > 0) {
  console.log(chalk.yellow(`⚠ Warnings: ${warnings}`));
}
if (errors > 0) {
  console.log(chalk.red(`✗ Errors: ${errors}`));
}
console.log(chalk.cyan(`Success Rate: ${percentage}%`));

// Recommendations
if (errors > 0 || warnings > 0) {
  console.log(chalk.bold('\n=== Recommendations ===\n'));

  if (errors > 0) {
    console.log(chalk.red('Critical issues found. Please install missing required tools:'));
    console.log(chalk.gray('  • Rust: https://rustup.rs/'));
    console.log(chalk.gray('  • Trunk: cargo install trunk'));
    console.log(chalk.gray('  • Node.js: https://nodejs.org/'));
  }

  if (warnings > 0) {
    console.log(chalk.yellow('\nOptional tools missing (recommended for full functionality):'));
    console.log(chalk.gray('  • WASM target: rustup target add wasm32-unknown-unknown'));
    console.log(chalk.gray('  • wasm-opt: Install binaryen'));
    console.log(chalk.gray('  • Wrangler: npm install -g wrangler'));
    console.log(chalk.gray('  • Dependencies: npm install'));
  }
}

// Next steps
console.log(chalk.bold('\n=== Next Steps ===\n'));

if (errors === 0) {
  console.log(chalk.green('✓ Environment ready!'));
  console.log(chalk.gray('\nQuick start:'));
  console.log(chalk.cyan('  1. npm install'));
  console.log(chalk.cyan('  2. npm run build:wasm'));
  console.log(chalk.cyan('  3. npm run dev'));
  console.log(chalk.gray('\nThen open: http://localhost:8080'));
} else {
  console.log(chalk.red('✗ Please fix errors before continuing'));
}

console.log();

// Exit with appropriate code
if (errors > 0) {
  process.exit(1);
} else if (warnings > 0) {
  process.exit(0); // Warnings are acceptable
} else {
  process.exit(0);
}
