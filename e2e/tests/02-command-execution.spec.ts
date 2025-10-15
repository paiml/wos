import { test, expect } from '@playwright/test';

test.describe('Command Execution', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('should execute help command', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Type help command
    await input.fill('help');
    await input.press('Enter');

    // Check output
    await expect(output).toContainText('Available commands');
    await expect(output).toContainText('help');
    await expect(output).toContainText('ps');
    await expect(output).toContainText('echo');
  });

  test('should execute echo command', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Type echo command
    await input.fill('echo hello world');
    await input.press('Enter');

    // Check output
    await expect(output).toContainText('hello world');
  });

  test('should execute version command', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Type version command
    await input.fill('version');
    await input.press('Enter');

    // Check output
    await expect(output).toContainText('WOS v');
    await expect(output).toContainText('kernel:');
    await expect(output).toContainText('userspace:');
  });

  test('should execute ps command', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Type ps command
    await input.fill('ps');
    await input.press('Enter');

    // Check output
    await expect(output).toContainText('PID');
    await expect(output).toContainText('STATE');
  });

  test('should execute state command', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Type state command
    await input.fill('state');
    await input.press('Enter');

    // Check output
    await expect(output).toContainText('Kernel State');
    await expect(output).toContainText('Processes:');
    await expect(output).toContainText('Next PID:');
  });

  test('should handle unknown command', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Type unknown command
    await input.fill('unknown_command');
    await input.press('Enter');

    // Check error message
    await expect(output).toContainText('Unknown command');
    await expect(output).toContainText('unknown_command');
  });

  test('should clear input after command execution', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Type and execute command
    await input.fill('help');
    await input.press('Enter');

    // Input should be empty
    await expect(input).toHaveValue('');
  });

  test('should execute multiple commands', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute first command
    await input.fill('echo first');
    await input.press('Enter');
    await expect(output).toContainText('first');

    // Execute second command
    await input.fill('echo second');
    await input.press('Enter');
    await expect(output).toContainText('second');

    // Execute third command
    await input.fill('echo third');
    await input.press('Enter');
    await expect(output).toContainText('third');

    // All outputs should be visible
    await expect(output).toContainText('first');
    await expect(output).toContainText('second');
    await expect(output).toContainText('third');
  });
});
