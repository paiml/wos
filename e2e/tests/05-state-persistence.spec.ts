import { test, expect } from '@playwright/test';

test.describe('State Persistence', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('should save state to localStorage', async ({ page }) => {
    const saveBtn = page.locator('#btn-save');
    const output = page.locator('#terminal-output');

    // Click save button
    await saveBtn.click();

    // Should show success message
    await expect(output).toContainText('State saved to localStorage');

    // Verify localStorage has the state
    const hasState = await page.evaluate(() => {
      return localStorage.getItem('wos-state') !== null;
    });

    expect(hasState).toBeTruthy();
  });

  test('should load state from localStorage', async ({ page }) => {
    // First, save some state
    const saveBtn = page.locator('#btn-save');
    await saveBtn.click();

    // Reload page
    await page.reload();
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

    // Load state
    const loadBtn = page.locator('#btn-load');
    const output = page.locator('#terminal-output');
    await loadBtn.click();

    // Should show success message
    await expect(output).toContainText('State loaded from localStorage');
  });

  test('should handle load with no saved state', async ({ page }) => {
    // Clear localStorage
    await page.evaluate(() => {
      localStorage.removeItem('wos-state');
    });

    const loadBtn = page.locator('#btn-load');
    const output = page.locator('#terminal-output');

    // Try to load
    await loadBtn.click();

    // Should show error message
    await expect(output).toContainText('No saved state found');
  });

  test('should preserve state after save and load', async ({ page }) => {
    const _input = page.locator('#terminal-input');
    const saveBtn = page.locator('#btn-save');
    const resetBtn = page.locator('#btn-reset');
    const loadBtn = page.locator('#btn-load');
    const processCount = page.locator('#process-count');

    // Get initial process count
    const initialCount = await processCount.textContent();

    // Save state
    await saveBtn.click();

    // Reset system
    await resetBtn.click();

    // Process count might change
    const _resetCount = await processCount.textContent();

    // Load state
    await loadBtn.click();

    // Wait for state to be restored
    await page.waitForTimeout(500);

    // Process count should match initial
    const restoredCount = await processCount.textContent();
    expect(restoredCount).toBe(initialCount);
  });
});
