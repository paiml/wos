import { test, expect } from '@playwright/test';

test.describe('Configuration Management', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto('index.html');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('should load default configuration on startup', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute config command to verify config is loaded
    await input.fill('config');
    await input.press('Enter');
    await page.waitForTimeout(500);

    // Verify config is displayed
    const outputText = await output.textContent();
    expect(outputText).toContain('Current Configuration');
    expect(outputText).toContain('Version:');
  });

  test('should display configuration with config command', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute config command
    await input.fill('config');
    await input.press('Enter');

    // Wait for output
    await page.waitForTimeout(500);

    // Check that config info is displayed
    const outputText = await output.textContent();
    expect(outputText).toContain('Current Configuration');
    expect(outputText).toContain('Version:');
    expect(outputText).toContain('Environment:');
    expect(outputText).toContain('UI Settings');
    expect(outputText).toContain('Theme:');
  });

  test('should switch to dark theme', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const body = page.locator('body');

    // Execute theme dark command
    await input.fill('theme dark');
    await input.press('Enter');

    // Wait for theme to apply
    await page.waitForTimeout(500);

    // Verify dark theme class is applied
    const hasDarkTheme = await body.evaluate((el) => el.classList.contains('theme-dark'));
    expect(hasDarkTheme).toBe(true);

    // Verify success message
    const output = await page.locator('#terminal-output').textContent();
    expect(output).toContain('Theme set to: dark');
  });

  test('should switch to light theme', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const body = page.locator('body');

    // Execute theme light command
    await input.fill('theme light');
    await input.press('Enter');

    // Wait for theme to apply
    await page.waitForTimeout(500);

    // Verify light theme class is applied
    const hasLightTheme = await body.evaluate((el) => el.classList.contains('theme-light'));
    expect(hasLightTheme).toBe(true);

    // Verify success message
    const output = await page.locator('#terminal-output').textContent();
    expect(output).toContain('Theme set to: light');
  });

  test('should switch to auto theme', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Execute theme auto command
    await input.fill('theme auto');
    await input.press('Enter');

    // Wait for theme to apply
    await page.waitForTimeout(500);

    // Verify success message
    const output = await page.locator('#terminal-output').textContent();
    expect(output).toContain('Theme set to: auto');

    // Auto theme should apply either dark or light based on system preference
    const body = page.locator('body');
    const hasDarkTheme = await body.evaluate((el) => el.classList.contains('theme-dark'));
    const hasLightTheme = await body.evaluate((el) => el.classList.contains('theme-light'));

    // Should have exactly one theme class
    expect(hasDarkTheme || hasLightTheme).toBe(true);
  });

  test('should persist theme setting in localStorage', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Set theme to light
    await input.fill('theme light');
    await input.press('Enter');
    await page.waitForTimeout(500);

    // Verify config is saved in localStorage
    const savedConfig = await page.evaluate(() => {
      return localStorage.getItem('wos-config');
    });
    expect(savedConfig).toBeTruthy();
    expect(savedConfig).toContain('light');

    // Reload page and verify theme persists
    await page.reload();
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

    const body = page.locator('body');
    const hasLightTheme = await body.evaluate((el) => el.classList.contains('theme-light'));
    expect(hasLightTheme).toBe(true);
  });

  test('should include config commands in help', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute help command
    await input.fill('help');
    await input.press('Enter');

    // Wait for output
    await page.waitForTimeout(500);

    // Check that config commands are listed
    const outputText = await output.textContent();
    expect(outputText).toContain('config');
    expect(outputText).toContain('theme dark');
    expect(outputText).toContain('theme light');
    expect(outputText).toContain('theme auto');
  });

  test('should handle rapid theme switches', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const body = page.locator('body');

    // Rapidly switch themes
    await input.fill('theme dark');
    await input.press('Enter');
    await page.waitForTimeout(100);

    await input.fill('theme light');
    await input.press('Enter');
    await page.waitForTimeout(100);

    await input.fill('theme dark');
    await input.press('Enter');
    await page.waitForTimeout(500);

    // Final theme should be dark
    const hasDarkTheme = await body.evaluate((el) => el.classList.contains('theme-dark'));
    expect(hasDarkTheme).toBe(true);
  });

  test('should maintain theme across command executions', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const body = page.locator('body');

    // Set theme to light
    await input.fill('theme light');
    await input.press('Enter');
    await page.waitForTimeout(500);

    // Execute other commands
    await input.fill('ps');
    await input.press('Enter');
    await page.waitForTimeout(300);

    await input.fill('ls');
    await input.press('Enter');
    await page.waitForTimeout(300);

    // Theme should still be light
    const hasLightTheme = await body.evaluate((el) => el.classList.contains('theme-light'));
    expect(hasLightTheme).toBe(true);
  });

  test('should show config details after theme change', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Change theme
    await input.fill('theme light');
    await input.press('Enter');
    await page.waitForTimeout(500);

    // Clear output for cleaner check
    await input.fill('clear');
    await input.press('Enter');
    await page.waitForTimeout(300);

    // Show config
    await input.fill('config');
    await input.press('Enter');
    await page.waitForTimeout(500);

    // Verify theme is shown as light
    const outputText = await output.textContent();
    expect(outputText).toContain('Theme: light');
  });
});
