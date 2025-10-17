import { test, expect } from '@playwright/test';

test.describe('UI Interactions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('should clear terminal with Ctrl+L', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute some commands
    await input.fill('echo test1');
    await input.press('Enter');
    await input.fill('echo test2');
    await input.press('Enter');

    // Verify output exists
    await expect(output).toContainText('test1');
    await expect(output).toContainText('test2');

    // Clear with Ctrl+L
    await input.press('Control+l');

    // Output should only have welcome message
    const text = await output.textContent();
    expect(text).not.toContain('test1');
    expect(text).not.toContain('test2');
    await expect(output).toContainText('WOS - WebAssembly Operating System');
  });

  test('should clear terminal with clear button', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');
    const clearBtn = page.locator('#btn-clear');

    // Execute a command
    await input.fill('echo test');
    await input.press('Enter');
    await expect(output).toContainText('test');

    // Click clear button
    await clearBtn.click();

    // Output should be cleared
    const text = await output.textContent();
    expect(text).not.toContain('wos$ echo test');
  });

  test('should reset system with reset button', async ({ page }) => {
    const resetBtn = page.locator('#btn-reset');
    const output = page.locator('#terminal-output');

    // Click reset button
    await resetBtn.click();

    // Should show success message
    await expect(output).toContainText('System reset successfully');
  });

  test('should keep input focused when clicking terminal', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const terminal = page.locator('#terminal');

    // Click on terminal area
    await terminal.click();

    // Input should be focused
    await expect(input).toBeFocused();
  });

  test('should auto-scroll to bottom on new output', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const terminal = page.locator('#terminal');

    // Execute multiple commands to create scrolling
    for (let i = 0; i < 20; i++) {
      await input.fill(`echo line ${i}`);
      await input.press('Enter');
    }

    // Terminal should be scrolled to bottom
    const isAtBottom = await terminal.evaluate((el) => {
      return Math.abs(el.scrollHeight - el.scrollTop - el.clientHeight) < 10;
    });

    expect(isAtBottom).toBeTruthy();
  });

  test('should display quality metrics', async ({ page }) => {
    await page.waitForSelector('#tdg-grade', { timeout: 10000 });

    const grade = page.locator('#tdg-grade');
    const score = page.locator('#tdg-score');
    const testCount = page.locator('#test-count');
    const coverage = page.locator('#coverage');

    // All quality metrics should be visible
    await expect(grade).toBeVisible();
    await expect(score).toBeVisible();
    await expect(testCount).toBeVisible();
    await expect(coverage).toBeVisible();

    // Grade should be A+ or A
    const gradeText = await grade.textContent();
    expect(gradeText).toMatch(/A\+?/);
  });

  test('should export quality metrics as JSON', async ({ page }) => {
    const exportBtn = page.locator('#btn-export-json');

    // Setup download listener
    const downloadPromise = page.waitForEvent('download');

    // Click export button
    await exportBtn.click();

    // Wait for download
    const download = await downloadPromise;

    // Verify filename
    expect(download.suggestedFilename()).toBe('wos-quality-metrics.json');
  });

  test('should export quality report as HTML', async ({ page }) => {
    const exportBtn = page.locator('#btn-export-html');

    // Setup download listener
    const downloadPromise = page.waitForEvent('download');

    // Click export button
    await exportBtn.click();

    // Wait for download
    const download = await downloadPromise;

    // Verify filename
    expect(download.suggestedFilename()).toBe('wos-quality-report.html');
  });
});
