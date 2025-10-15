import { test, expect, Page } from '@playwright/test';

/**
 * Canary Test Suite - Terminal Interaction (C01-C09)
 *
 * SQLite Principle: Run before every release, every platform
 * WOS Adaptation: Run before every deploy, every browser
 *
 * Coverage: 20% of user actions (terminal interaction)
 * Category: Critical user workflows
 * Performance Target: <100ms per command
 */

// Helper functions for common operations
async function executeCommand(page: Page, command: string): Promise<void> {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  // Wait for command to complete
  await page.waitForTimeout(50);
}

async function getLastOutput(page: Page): Promise<string> {
  const output = page.locator('#terminal-output');
  const text = await output.textContent();
  return text || '';
}

async function getCommandResponseTime(page: Page, command: string): Promise<number> {
  const input = page.locator('#terminal-input');
  const startTime = Date.now();

  await input.fill(command);
  await input.press('Enter');

  // Wait for output to appear
  const output = page.locator('#terminal-output');
  await output.waitFor({ state: 'visible', timeout: 1000 });

  const endTime = Date.now();
  return endTime - startTime;
}

test.describe('Canary: Terminal Interaction', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('');
    // Increased timeout for WASM initialization
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });
  });

  test('C01: User types command and sees output', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Type command
    await input.fill('echo hello world');
    await input.press('Enter');

    // Verify output appears
    await expect(output).toContainText('hello world', { timeout: 100 });

    // Verify prompt returns and input is enabled
    await expect(input).toBeEnabled();
    await expect(input).toHaveValue('');
  });

  test('C02: Command history with arrow keys', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Execute multiple commands
    await executeCommand(page, 'echo first');
    await executeCommand(page, 'echo second');
    await executeCommand(page, 'echo third');

    // Navigate history backward
    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo third');

    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo second');

    await input.press('ArrowUp');
    await expect(input).toHaveValue('echo first');

    // Navigate history forward
    await input.press('ArrowDown');
    await expect(input).toHaveValue('echo second');

    await input.press('ArrowDown');
    await expect(input).toHaveValue('echo third');

    // At end of history, input should clear
    await input.press('ArrowDown');
    await expect(input).toHaveValue('');
  });

  test('C03: Clear terminal with Ctrl+L', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute some commands to populate terminal
    await executeCommand(page, 'echo line1');
    await executeCommand(page, 'echo line2');
    await executeCommand(page, 'echo line3');

    // Verify output exists
    await expect(output).toContainText('line1');
    await expect(output).toContainText('line2');

    // Clear terminal
    await input.press('Control+L');

    // Verify terminal cleared (output should be empty or minimal)
    const outputText = await output.textContent();
    expect(outputText).not.toContain('line1');
    expect(outputText).not.toContain('line2');
  });

  test('C04: Multiple commands in rapid succession', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute 10 commands rapidly
    const commands = [
      'echo test1',
      'echo test2',
      'echo test3',
      'echo test4',
      'echo test5',
      'ps',
      'version',
      'echo test6',
      'echo test7',
      'echo test8'
    ];

    for (const cmd of commands) {
      await input.fill(cmd);
      await input.press('Enter');
      // Minimal wait between commands
      await page.waitForTimeout(10);
    }

    // Verify all outputs appear
    const outputText = await getLastOutput(page);
    expect(outputText).toContain('test1');
    expect(outputText).toContain('test8');
    expect(outputText).toContain('PID'); // from ps
    expect(outputText).toContain('WOS v'); // from version
  });

  test('C05: Long output scrolling performance', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Generate long output with help command repeated
    for (let i = 0; i < 20; i++) {
      await executeCommand(page, 'help');
    }

    // Verify terminal still responsive
    const responseTime = await getCommandResponseTime(page, 'echo scroll test');
    expect(responseTime).toBeLessThan(200); // Should still respond quickly

    // Verify output contains the test echo
    await expect(output).toContainText('scroll test');
  });

  test('C06: Special characters in commands', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Test various special characters
    const specialCases = [
      { cmd: 'echo "quoted text"', expected: 'quoted text' },
      { cmd: 'echo test123', expected: 'test123' },
      { cmd: 'echo test_underscore', expected: 'test_underscore' },
      { cmd: 'echo test-dash', expected: 'test-dash' },
      { cmd: 'echo test.dot', expected: 'test.dot' },
    ];

    for (const { cmd, expected } of specialCases) {
      await executeCommand(page, cmd);
      const outputText = await getLastOutput(page);
      expect(outputText).toContain(expected);
    }
  });

  test('C07: Command input validation', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Test empty command (should be ignored)
    await input.fill('');
    await input.press('Enter');
    await page.waitForTimeout(50);

    // Input should still be enabled and empty
    await expect(input).toBeEnabled();
    await expect(input).toHaveValue('');

    // Test whitespace-only command
    await input.fill('   ');
    await input.press('Enter');
    await page.waitForTimeout(50);

    // Should handle gracefully
    await expect(input).toBeEnabled();

    // Test valid command after invalid ones
    await executeCommand(page, 'echo valid');
    await expect(output).toContainText('valid');
  });

  test('C08: Error message display', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute invalid command
    await executeCommand(page, 'invalidcommand123');

    // Verify error message appears
    const outputText = await getLastOutput(page);
    expect(outputText.toLowerCase()).toMatch(/unknown|not found|invalid/);

    // Verify terminal still responsive after error
    await executeCommand(page, 'echo recovery test');
    await expect(output).toContainText('recovery test');
  });

  test('C09: Terminal state consistency', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Perform various operations
    await executeCommand(page, 'echo test1');
    await executeCommand(page, 'ps');
    await executeCommand(page, 'invalidcommand');
    await executeCommand(page, 'version');

    // Clear terminal
    await input.press('Control+L');

    // Execute new command
    await executeCommand(page, 'echo after clear');

    // Navigate history
    await input.press('ArrowUp');
    const historyValue = await input.inputValue();

    // History should contain recent command
    expect(historyValue).toBeTruthy();
    expect(historyValue.length).toBeGreaterThan(0);

    // Terminal should be fully functional
    await executeCommand(page, 'echo final test');
    await expect(output).toContainText('final test');
    await expect(input).toBeEnabled();
  });

  test('C09B: Performance baseline - command response time', async ({ page }) => {
    // Measure response time for various commands
    const commands = [
      { cmd: 'echo test', maxTime: 100 },
      { cmd: 'ps', maxTime: 150 },
      { cmd: 'version', maxTime: 100 },
      { cmd: 'help', maxTime: 150 },
    ];

    for (const { cmd, maxTime } of commands) {
      const responseTime = await getCommandResponseTime(page, cmd);
      expect(responseTime).toBeLessThan(maxTime);
      console.log(`${cmd}: ${responseTime}ms (target: <${maxTime}ms)`);
    }
  });
});

test.describe('Canary: Terminal Interaction - Edge Cases', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('');
    // Increased timeout for WASM initialization
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });
  });

  test('C10: Very long command input', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Generate very long command (500 characters)
    const longText = 'a'.repeat(500);
    const command = `echo ${longText}`;

    await input.fill(command);
    await input.press('Enter');
    await page.waitForTimeout(200);

    // Should handle gracefully (either execute or reject gracefully)
    await expect(input).toBeEnabled();
  });

  test('C11: Rapid keyboard input', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Type characters rapidly
    const text = 'echo rapid input test';
    for (const char of text) {
      await input.press(char);
      // No delay between keypresses
    }

    await input.press('Enter');
    await page.waitForTimeout(100);

    // Terminal should still be responsive
    await expect(input).toBeEnabled();
  });

  test('C12: Command history boundary conditions', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Execute exactly one command
    await executeCommand(page, 'echo single');

    // Try to navigate beyond history
    await input.press('ArrowUp'); // Should show 'echo single'
    await expect(input).toHaveValue('echo single');

    await input.press('ArrowUp'); // Should stay at first command
    await expect(input).toHaveValue('echo single');

    await input.press('ArrowDown'); // Should clear
    await expect(input).toHaveValue('');

    await input.press('ArrowDown'); // Should stay empty
    await expect(input).toHaveValue('');
  });
});
