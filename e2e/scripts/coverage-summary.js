#!/usr/bin/env node
/**
 * E2E Test Coverage Summary Generator
 * Generates a summary report of E2E test coverage including test counts and pass rates
 */

const fs = require('fs');
const path = require('path');

const RESULTS_FILE = path.join(__dirname, '../test-results.json');
const COVERAGE_DIR = path.join(__dirname, '../../target/coverage');

function main() {
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('📊 E2E Test Coverage Summary');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('');

  // Check if results file exists
  if (!fs.existsSync(RESULTS_FILE)) {
    console.log('⚠️  No test results found. Run `npm run test:coverage` first.');
    console.log('');
    return;
  }

  const results = JSON.parse(fs.readFileSync(RESULTS_FILE, 'utf8'));

  // Recursively collect all specs from nested suites
  function collectSpecs(suite) {
    const specs = [];
    if (suite.specs && suite.specs.length > 0) {
      specs.push(...suite.specs);
    }
    if (suite.suites && suite.suites.length > 0) {
      for (const nestedSuite of suite.suites) {
        specs.push(...collectSpecs(nestedSuite));
      }
    }
    return specs;
  }

  // Calculate statistics
  const allSpecs = results.suites.flatMap(collectSpecs);
  const stats = allSpecs.reduce((acc, spec) => {
    acc.total++;
    if (spec.ok) {
      acc.passed++;
    } else {
      acc.failed++;
    }
    if (spec.tests[0]?.status === 'skipped') {
      acc.skipped++;
    }
    return acc;
  }, { total: 0, passed: 0, failed: 0, skipped: 0 });

  const passRate = stats.total > 0 ? (stats.passed / stats.total * 100).toFixed(2) : 0;

  console.log(`Total Tests:    ${stats.total}`);
  console.log(`✅ Passed:      ${stats.passed}`);
  console.log(`❌ Failed:      ${stats.failed}`);
  console.log(`⏭️  Skipped:     ${stats.skipped}`);
  console.log(`📈 Pass Rate:   ${passRate}%`);
  console.log('');

  // List test suites
  console.log('Test Suites:');
  results.suites.forEach(suite => {
    const suiteSpecs = collectSpecs(suite);
    const suitePassed = suiteSpecs.filter(s => s.ok).length;
    const suiteTotal = suiteSpecs.length;
    const suiteName = path.basename(suite.file);
    console.log(`  ${suitePassed === suiteTotal ? '✅' : '❌'} ${suiteName}: ${suitePassed}/${suiteTotal} passed`);
  });
  console.log('');

  // Save summary to coverage directory
  if (!fs.existsSync(COVERAGE_DIR)) {
    fs.mkdirSync(COVERAGE_DIR, { recursive: true });
  }

  const summary = {
    type: 'e2e',
    timestamp: new Date().toISOString(),
    stats,
    passRate: parseFloat(passRate),
    suites: results.suites.map(suite => {
      const suiteSpecs = collectSpecs(suite);
      return {
        file: path.basename(suite.file),
        passed: suiteSpecs.filter(s => s.ok).length,
        total: suiteSpecs.length
      };
    })
  };

  const summaryPath = path.join(COVERAGE_DIR, 'e2e-summary.json');
  fs.writeFileSync(summaryPath, JSON.stringify(summary, null, 2));
  console.log(`💾 Summary saved to: ${summaryPath}`);
  console.log('');
}

main();
