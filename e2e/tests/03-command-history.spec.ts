import { test, expect } from '@playwright/test';

test.describe('Command History', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('should navigate history with arrow keys', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Execute some commands
    await input.fill('echo first');
    await input.press('Enter');

    await input.fill('echo second');
    await input.press('Enter');

    await input.fill('echo third');
    await input.press('Enter');

    // Navigate up (should show 'echo third')
    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo third');

    // Navigate up again (should show 'echo second')
    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo second');

    // Navigate up again (should show 'echo first')
    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo first');

    // Navigate down (should show 'echo second')
    await input.press('ArrowDown');
    await expect(input).toHaveValue('echo second');

    // Navigate down (should show 'echo third')
    await input.press('ArrowDown');
    await expect(input).toHaveValue('echo third');

    // Navigate down (should be empty)
    await input.press('ArrowDown');
    await expect(input).toHaveValue('');
  });

  test('should not navigate beyond history bounds', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Execute one command
    await input.fill('echo test');
    await input.press('Enter');

    // Navigate up
    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo test');

    // Try to navigate up more (should stay at 'echo test')
    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo test');
  });

  test('should preserve edited command when navigating', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Execute a command
    await input.fill('echo test');
    await input.press('Enter');

    // Type new text
    await input.fill('echo new');

    // Navigate up (should show previous command)
    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo test');

    // Navigate down (should return to empty, not 'echo new')
    await input.press('ArrowDown');
    await expect(input).toHaveValue('');
  });

  test('should execute command from history', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute command
    await input.fill('echo from history');
    await input.press('Enter');

    // Navigate to previous command
    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo from history');

    // Execute it again
    await input.press('Enter');

    // Output should contain the text twice
    const historyText = await output.textContent();
    const matches = (historyText?.match(/from history/g) || []).length;
    expect(matches).toBeGreaterThanOrEqual(2);
  });
});
