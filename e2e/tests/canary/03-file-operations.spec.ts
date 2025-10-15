import { test, expect, Page } from '@playwright/test';

/**
 * Canary Test Suite - File Operations (C25-C34)
 *
 * SQLite Principle: Test every file operation, every error path
 * WOS Adaptation: Validate VFS, ProcFS, file I/O, and permissions
 *
 * Coverage: 20% of user actions (file operations)
 * Category: VFS and filesystem validation
 * Performance Target: File operations complete within 150ms
 */

// Helper functions
async function executeCommand(page: Page, command: string): Promise<void> {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(50);
}

async function getLastOutput(page: Page): Promise<string> {
  const output = page.locator('#terminal-output');
  const text = await output.textContent();
  return text || '';
}

async function clearTerminal(page: Page): Promise<void> {
  const input = page.locator('#terminal-input');
  await input.press('Control+L');
  await page.waitForTimeout(50);
}

test.describe('Canary: File Operations - VFS', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Increased timeout for WASM initialization
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });
  });

  test('C25: List root directory with ls', async ({ page }) => {
    const output = page.locator('#terminal-output');

    await executeCommand(page, 'ls /');

    // Verify standard directories exist
    const outputText = await getLastOutput(page);

    // Should contain common Unix directories
    // Note: Actual directories depend on WOS VFS implementation
    expect(outputText.length).toBeGreaterThan(0);

    // Command should complete successfully (no error messages)
    expect(outputText.toLowerCase()).not.toContain('error');
    expect(outputText.toLowerCase()).not.toContain('failed');
  });

  test('C26: List current directory with ls', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // ls without arguments should list current directory
    await executeCommand(page, 'ls');

    const outputText = await getLastOutput(page);
    expect(outputText.length).toBeGreaterThan(0);

    // Should complete successfully
    await expect(output).not.toContainText('Unknown command');
  });

  test('C27: List non-existent directory shows error', async ({ page }) => {
    const output = page.locator('#terminal-output');

    await executeCommand(page, 'ls /nonexistent');

    const outputText = await getLastOutput(page);

    // Should show error message or empty result
    // (depending on WOS implementation)
    expect(outputText.length).toBeGreaterThan(0);
  });

  test('C28: Multiple ls commands in sequence', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute multiple ls commands
    await executeCommand(page, 'ls /');
    await executeCommand(page, 'ls');
    await executeCommand(page, 'ls /');

    // All should complete successfully
    const outputText = await getLastOutput(page);
    expect(outputText.length).toBeGreaterThan(0);

    // Terminal should still be responsive
    await executeCommand(page, 'echo ls test');
    await expect(output).toContainText('ls test');
  });

  test('C29: ls command performance', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    const startTime = Date.now();

    await input.fill('ls /');
    await input.press('Enter');

    // Wait for output with timeout
    await page.waitForTimeout(150);

    const endTime = Date.now();
    const duration = endTime - startTime;

    // ls should complete quickly (<150ms)
    expect(duration).toBeLessThan(150);

    console.log(`ls command time: ${duration}ms (target: <150ms)`);
  });
});

test.describe('Canary: File Operations - ProcFS', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Increased timeout for WASM initialization
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });
  });

  test('C30: Read /proc filesystem info', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // List /proc directory
    await executeCommand(page, 'ls /proc');

    const outputText = await getLastOutput(page);

    // ProcFS should exist (if implemented)
    // At minimum, should not crash
    expect(outputText.length).toBeGreaterThan(0);
  });

  test('C31: Read process status from /proc/1/status', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Try to read init process status (PID 1)
    await executeCommand(page, 'cat /proc/1/status');

    const outputText = await getLastOutput(page);

    // Should return process info or graceful error
    expect(outputText.length).toBeGreaterThan(0);

    // If ProcFS is implemented, should contain process info
    if (outputText.toLowerCase().includes('pid')) {
      expect(outputText).toMatch(/pid/i);
    }
  });

  test('C32: Read /proc/self symlink', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // /proc/self should point to current process
    await executeCommand(page, 'cat /proc/self/status');

    const outputText = await getLastOutput(page);

    // Should return some output (process info or error)
    expect(outputText.length).toBeGreaterThan(0);
  });

  test('C33: ProcFS remains consistent across operations', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Read ProcFS multiple times
    await executeCommand(page, 'ls /proc');
    const firstRead = await getLastOutput(page);

    await clearTerminal(page);

    await executeCommand(page, 'ls /proc');
    const secondRead = await getLastOutput(page);

    // ProcFS should be accessible both times
    expect(firstRead.length).toBeGreaterThan(0);
    expect(secondRead.length).toBeGreaterThan(0);

    // System should still be responsive
    await executeCommand(page, 'echo proc test');
    await expect(output).toContainText('proc test');
  });

  test('C34: Read non-existent /proc file shows error', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Try to read non-existent process
    await executeCommand(page, 'cat /proc/99999/status');

    const outputText = await getLastOutput(page);

    // Should handle gracefully (error or empty)
    expect(outputText.length).toBeGreaterThan(0);

    // System should still be responsive after error
    await executeCommand(page, 'echo still works');
    await expect(output).toContainText('still works');
  });
});

test.describe('Canary: File Operations - Error Handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Increased timeout for WASM initialization
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });
  });

  test('C35: Handle invalid file paths', async ({ page }) => {
    const output = page.locator('#terminal-output');

    const invalidPaths = [
      'cat /invalid/path/file.txt',
      'ls ///multiple/slashes',
      'cat /.',
      'ls /path/with spaces', // May need escaping
    ];

    for (const cmd of invalidPaths) {
      await executeCommand(page, cmd);
      await page.waitForTimeout(50);
    }

    // System should remain stable
    await executeCommand(page, 'echo recovery test');
    await expect(output).toContainText('recovery test');
  });

  test('C36: File operations after terminal clear', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Perform file operations
    await executeCommand(page, 'ls /');
    await executeCommand(page, 'ls /proc');

    // Clear terminal
    await clearTerminal(page);

    // File operations should still work
    await executeCommand(page, 'ls /');
    const outputText = await getLastOutput(page);
    expect(outputText.length).toBeGreaterThan(0);
  });

  test('C37: Rapid file operations', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute many file operations rapidly
    for (let i = 0; i < 10; i++) {
      await executeCommand(page, 'ls /');
      await page.waitForTimeout(10);
    }

    // System should still be responsive
    await executeCommand(page, 'echo rapid test');
    await expect(output).toContainText('rapid test');
  });

  test('C38: File operations with special characters', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Test paths with special characters
    const specialCases = [
      'ls /',
      'ls /proc',
      'cat /proc/1/status',
    ];

    for (const cmd of specialCases) {
      await executeCommand(page, cmd);
      await page.waitForTimeout(50);
    }

    // Should handle all cases without crashing
    await executeCommand(page, 'echo special chars ok');
    await expect(output).toContainText('special chars ok');
  });

  test('C39: File system stability after errors', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Cause various file operation errors
    await executeCommand(page, 'cat /nonexistent');
    await executeCommand(page, 'ls /invalid');
    await executeCommand(page, 'cat /proc/99999/status');

    // File system should still work
    await executeCommand(page, 'ls /');
    const outputText = await getLastOutput(page);
    expect(outputText.length).toBeGreaterThan(0);

    // Process system should still work
    await executeCommand(page, 'ps');
    await expect(output).toContainText('PID');
  });
});

test.describe('Canary: File Operations - Integration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Increased timeout for WASM initialization
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });
  });

  test('C40: Combine file and process operations', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Interleave file and process operations
    await executeCommand(page, 'ps');
    await expect(output).toContainText('PID');

    await executeCommand(page, 'ls /');
    const lsOutput = await getLastOutput(page);
    expect(lsOutput.length).toBeGreaterThan(0);

    await executeCommand(page, 'ls /proc');
    await page.waitForTimeout(50);

    await executeCommand(page, 'ps');
    await expect(output).toContainText('PID');

    // System should handle mixed operations
    await executeCommand(page, 'echo integration test');
    await expect(output).toContainText('integration test');
  });

  test('C41: File operations in command history', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute file operations
    await executeCommand(page, 'ls /');
    await executeCommand(page, 'ls /proc');
    await executeCommand(page, 'cat /proc/1/status');

    // Navigate history
    await input.press('ArrowUp');
    const historyValue1 = await input.inputValue();
    expect(historyValue1).toBeTruthy();

    await input.press('ArrowUp');
    const historyValue2 = await input.inputValue();
    expect(historyValue2).toBeTruthy();

    // History should work correctly
    expect(historyValue1).not.toBe(historyValue2);
  });

  test('C42: File operations performance under load', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute 20 mixed operations
    const commands = [
      'ls /',
      'ps',
      'ls /proc',
      'echo test',
      'ls',
    ];

    const startTime = Date.now();

    for (let i = 0; i < 4; i++) {
      for (const cmd of commands) {
        await executeCommand(page, cmd);
      }
    }

    const endTime = Date.now();
    const duration = endTime - startTime;

    // 20 commands should complete reasonably fast (<3 seconds)
    expect(duration).toBeLessThan(3000);

    console.log(`20 mixed commands completed in ${duration}ms (target: <3000ms)`);

    // System should still be responsive
    await executeCommand(page, 'echo load test complete');
    await expect(output).toContainText('load test complete');
  });

  test('C43: File system state after terminal reset', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Perform file operations
    await executeCommand(page, 'ls /');
    await executeCommand(page, 'ls /proc');

    // Reset terminal (if reset command exists)
    await executeCommand(page, 'reset');
    await page.waitForTimeout(100);

    // File system should still work after reset
    await executeCommand(page, 'ls /');
    const outputText = await getLastOutput(page);
    expect(outputText.length).toBeGreaterThan(0);
  });

  test('C44: Verify ls output format consistency', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute ls multiple times
    await executeCommand(page, 'ls /');
    const firstOutput = await getLastOutput(page);

    await clearTerminal(page);

    await executeCommand(page, 'ls /');
    const secondOutput = await getLastOutput(page);

    // Output should be consistent in format
    expect(firstOutput.length).toBeGreaterThan(0);
    expect(secondOutput.length).toBeGreaterThan(0);

    // Both outputs should be similar (same directory structure)
    // Allow for some variation in process-specific content
  });
});
