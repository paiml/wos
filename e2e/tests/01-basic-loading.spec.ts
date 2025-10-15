import { test, expect } from '@playwright/test';

test.describe('Basic Loading', () => {
  test('should load the WOS application', async ({ page }) => {
    await page.goto('/');

    // Wait for page to load
    await expect(page).toHaveTitle(/WOS/);

    // Check for terminal element
    const terminal = page.locator('#terminal');
    await expect(terminal).toBeVisible();
  });

  test('should display welcome message', async ({ page }) => {
    await page.goto('/');

    // Wait for WASM to initialize
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

    // Check for welcome message
    const output = page.locator('#terminal-output');
    await expect(output).toContainText('WOS - WebAssembly Operating System');
    await expect(output).toContainText('Type "help" for available commands');
  });

  test('should load WASM successfully', async ({ page }) => {
    await page.goto('/');

    // Wait for WASM initialization
    const status = page.locator('#status');
    await expect(status).toHaveText('Ready', { timeout: 10000 });

    // Check version is displayed
    const version = page.locator('#version');
    await expect(version).toContainText('WOS v');
  });

  test('should have input field focused', async ({ page }) => {
    await page.goto('/');

    // Wait for page to load
    await page.waitForSelector('#terminal-input');

    // Input should be focused
    const input = page.locator('#terminal-input');
    await expect(input).toBeFocused();
  });

  test('should display process count', async ({ page }) => {
    await page.goto('/');

    // Wait for initialization
    await page.waitForSelector('#status:has-text("Ready")');

    // Process count should be visible
    const processCount = page.locator('#process-count');
    await expect(processCount).toBeVisible();
  });
});
