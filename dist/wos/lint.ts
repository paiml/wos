#!/usr/bin/env -S deno run --allow-read --allow-write

/**
 * WOS Frontend Linting Script
 *
 * Validates HTML, CSS, and JavaScript files for:
 * - Syntax errors
 * - Style consistency
 * - Best practices
 * - Accessibility
 */

interface LintResult {
  file: string;
  issues: LintIssue[];
  warnings: LintIssue[];
}

interface LintIssue {
  line?: number;
  column?: number;
  message: string;
  severity: 'error' | 'warning' | 'info';
}

class FrontendLinter {
  private results: LintResult[] = [];
  private totalErrors = 0;
  private totalWarnings = 0;

  async lintHTML(file: string): Promise<LintResult> {
    const content = await Deno.readTextFile(file);
    const issues: LintIssue[] = [];
    const warnings: LintIssue[] = [];

    // Check for basic HTML structure
    if (!content.includes('<!DOCTYPE html>')) {
      issues.push({
        line: 1,
        message: 'Missing DOCTYPE declaration',
        severity: 'error',
      });
    }

    // Check for lang attribute
    if (!content.match(/<html[^>]+lang=/)) {
      issues.push({
        message: 'Missing lang attribute on <html>',
        severity: 'error',
      });
    }

    // Check for viewport meta tag
    if (!content.includes('viewport')) {
      warnings.push({
        message: 'Missing viewport meta tag',
        severity: 'warning',
      });
    }

    // Check for semantic HTML5 tags
    const semanticTags = ['header', 'main', 'footer', 'nav', 'section', 'article'];
    let hasSemantic = false;
    for (const tag of semanticTags) {
      if (content.includes(`<${tag}`)) {
        hasSemantic = true;
        break;
      }
    }
    if (!hasSemantic) {
      warnings.push({
        message: 'No semantic HTML5 tags found',
        severity: 'warning',
      });
    }

    // Check for accessibility - alt attributes
    const imgTags = content.match(/<img[^>]+>/g) || [];
    for (const img of imgTags) {
      if (!img.includes('alt=')) {
        issues.push({
          message: `Image missing alt attribute: ${img.substring(0, 50)}...`,
          severity: 'error',
        });
      }
    }

    // Check for inline styles (anti-pattern)
    const inlineStyles = content.match(/style="/g) || [];
    if (inlineStyles.length > 0) {
      warnings.push({
        message: `Found ${inlineStyles.length} inline style attributes`,
        severity: 'warning',
      });
    }

    // Check for script tags without type module
    const scriptTags = content.match(/<script[^>]+src=/g) || [];
    for (const script of scriptTags) {
      if (!script.includes('type="module"')) {
        warnings.push({
          message: 'Script tag should use type="module"',
          severity: 'warning',
        });
      }
    }

    return { file, issues, warnings };
  }

  async lintCSS(file: string): Promise<LintResult> {
    const content = await Deno.readTextFile(file);
    const issues: LintIssue[] = [];
    const warnings: LintIssue[] = [];
    const lines = content.split('\n');

    // Check for !important overuse
    const importantCount = (content.match(/!important/g) || []).length;
    if (importantCount > 5) {
      warnings.push({
        message: `Excessive use of !important (${importantCount} occurrences)`,
        severity: 'warning',
      });
    }

    // Check for browser prefixes (should use autoprefixer)
    const prefixes = ['-webkit-', '-moz-', '-ms-', '-o-'];
    for (let i = 0; i < lines.length; i++) {
      for (const prefix of prefixes) {
        if (lines[i].includes(prefix)) {
          warnings.push({
            line: i + 1,
            message: `Manual browser prefix found: ${prefix}`,
            severity: 'warning',
          });
        }
      }
    }

    // Check for color contrast (basic hex check)
    const hexColors = content.match(/#[0-9a-f]{3,6}/gi) || [];
    const uniqueColors = [...new Set(hexColors)];
    if (uniqueColors.length > 20) {
      warnings.push({
        message: `Many unique colors (${uniqueColors.length}) - consider using CSS variables`,
        severity: 'warning',
      });
    }

    // Check for CSS variables usage
    if (!content.includes('--')) {
      warnings.push({
        message: 'No CSS custom properties (variables) found',
        severity: 'info',
      });
    }

    // Check for missing vendor prefixes on flexbox
    if (content.includes('display: flex') && !content.includes('display: -webkit-flex')) {
      // This is actually OK with modern browsers, just informational
    }

    // Check for font-size in px (should use rem)
    const pxFontSizes = content.match(/font-size:\s*\d+px/g) || [];
    if (pxFontSizes.length > 0) {
      warnings.push({
        message: `Found ${pxFontSizes.length} font-size declarations in px - consider using rem`,
        severity: 'info',
      });
    }

    return { file, issues, warnings };
  }

  async lintJavaScript(file: string): Promise<LintResult> {
    const content = await Deno.readTextFile(file);
    const issues: LintIssue[] = [];
    const warnings: LintIssue[] = [];
    const lines = content.split('\n');

    // Check for console.log (should be removed in production)
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes('console.log') && !lines[i].trim().startsWith('//')) {
        warnings.push({
          line: i + 1,
          message: 'console.log found - remove before production',
          severity: 'warning',
        });
      }
    }

    // Check for debugger statements
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes('debugger')) {
        issues.push({
          line: i + 1,
          message: 'debugger statement found',
          severity: 'error',
        });
      }
    }

    // Check for var usage (should use let/const)
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].match(/\bvar\s+/)) {
        warnings.push({
          line: i + 1,
          message: 'Use let or const instead of var',
          severity: 'warning',
        });
      }
    }

    // Check for == instead of ===
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].match(/[^=!]={2}[^=]/)) {
        warnings.push({
          line: i + 1,
          message: 'Use === instead of ==',
          severity: 'warning',
        });
      }
    }

    // Check for TODO/FIXME
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].match(/\/\/\s*(TODO|FIXME|HACK)/)) {
        issues.push({
          line: i + 1,
          message: 'Technical debt marker found',
          severity: 'error',
        });
      }
    }

    return { file, issues, warnings };
  }

  printResults() {
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('  WOS Frontend Lint Results');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

    for (const result of this.results) {
      if (result.issues.length === 0 && result.warnings.length === 0) {
        console.log(`✓ ${result.file} - No issues`);
      } else {
        console.log(`\n${result.file}:`);

        for (const issue of result.issues) {
          this.totalErrors++;
          const location = issue.line ? `:${issue.line}` : '';
          console.log(`  ❌ ${issue.message}${location}`);
        }

        for (const warning of result.warnings) {
          this.totalWarnings++;
          const location = warning.line ? `:${warning.line}` : '';
          const icon = warning.severity === 'info' ? 'ℹ️' : '⚠️';
          console.log(`  ${icon}  ${warning.message}${location}`);
        }
      }
    }

    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log(`  Errors: ${this.totalErrors}`);
    console.log(`  Warnings: ${this.totalWarnings}`);
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

    if (this.totalErrors > 0) {
      console.log('❌ Linting failed with errors');
      Deno.exit(1);
    } else if (this.totalWarnings > 0) {
      console.log('⚠️  Linting passed with warnings');
      Deno.exit(0);
    } else {
      console.log('✅ All files passed linting');
      Deno.exit(0);
    }
  }

  async run() {
    console.log('🔍 Linting frontend files...\n');

    // Lint HTML
    const htmlResult = await this.lintHTML('index.html');
    this.results.push(htmlResult);

    // Lint CSS
    const cssResult = await this.lintCSS('style.css');
    this.results.push(cssResult);

    // Lint JavaScript
    const jsResult = await this.lintJavaScript('app.js');
    this.results.push(jsResult);

    this.printResults();
  }
}

// Run linter
if (import.meta.main) {
  const linter = new FrontendLinter();
  await linter.run();
}
