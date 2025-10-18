import { test, expect } from '@playwright/test';

test.describe('UI Interactions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    // Clear localStorage to ensure clean state
    await page.evaluate(() => localStorage.clear());
    await page.reload();

    // Wait for WASM to initialize by checking #status element exists with "Ready" text
    // Note: #status might be in a collapsed panel, so we don't check visibility
    await page.waitForFunction(
      () => {
        const status = document.getElementById('status');
        return status && status.textContent === 'Ready';
      },
      { timeout: 10000 }
    );
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

    // Wait for DOM updates to complete (instant scroll with scroll-behavior: auto)
    await page.waitForTimeout(100);

    // Terminal should be scrolled to bottom
    const isAtBottom = await terminal.evaluate((el) => {
      return Math.abs(el.scrollHeight - el.scrollTop - el.clientHeight) < 10;
    });

    expect(isAtBottom).toBeTruthy();
  });
});
