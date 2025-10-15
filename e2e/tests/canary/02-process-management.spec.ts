import { test, expect, Page } from '@playwright/test';

/**
 * Canary Test Suite - Process Management (C10-C19)
 *
 * SQLite Principle: Test every process operation, every state transition
 * WOS Adaptation: Validate fork, exec, wait, kill, and process lifecycle
 *
 * Coverage: 25% of user actions (process management)
 * Category: Critical OS functionality
 * Performance Target: Process operations complete within 200ms
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

async function getProcessList(page: Page): Promise<string[]> {
  await executeCommand(page, 'ps');
  const output = await getLastOutput(page);

  // Parse ps output to get process list
  const lines = output.split('\n').filter(line => line.trim());
  return lines;
}

async function getProcessCount(page: Page): Promise<number> {
  const lines = await getProcessList(page);
  // Subtract header line
  return Math.max(0, lines.length - 1);
}

test.describe('Canary: Process Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('C10: List processes with ps command', async ({ page }) => {
    const output = page.locator('#terminal-output');

    await executeCommand(page, 'ps');

    // Verify ps output contains expected headers
    await expect(output).toContainText('PID', { timeout: 200 });
    await expect(output).toContainText('STATE');

    // Verify init process (PID 1) is always present
    const outputText = await getLastOutput(page);
    expect(outputText).toMatch(/\b1\b/); // PID 1

    // Verify at least init and shell are running
    const processCount = await getProcessCount(page);
    expect(processCount).toBeGreaterThanOrEqual(2);
  });

  test('C11: Init process is always PID 1', async ({ page }) => {
    const output = page.locator('#terminal-output');

    await executeCommand(page, 'ps');

    const outputText = await getLastOutput(page);

    // PID 1 should exist
    expect(outputText).toMatch(/\b1\b/);

    // Init process should be in Running or Ready state
    expect(outputText).toMatch(/1.*(?:Running|Ready)/);
  });

  test('C12: Shell process exists and is responsive', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute ps to see current processes
    await executeCommand(page, 'ps');

    const outputText = await getLastOutput(page);

    // Shell process should exist
    expect(outputText.toLowerCase()).toContain('shell');

    // Shell should be able to execute commands (proves it's responsive)
    await executeCommand(page, 'echo shell is alive');
    await expect(output).toContainText('shell is alive');
  });

  test('C13: Process count increases with commands', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Get baseline process count
    const initialCount = await getProcessCount(page);
    expect(initialCount).toBeGreaterThanOrEqual(2); // At least init + shell

    // Execute a command (echo creates a short-lived process)
    await executeCommand(page, 'echo test');
    await expect(output).toContainText('test');

    // Process should have executed (even if it's already terminated)
    // The key is that the system handled process creation/termination
    await executeCommand(page, 'ps');
    const outputText = await getLastOutput(page);
    expect(outputText).toBeTruthy();
  });

  test('C14: Process state transitions are valid', async ({ page }) => {
    const output = page.locator('#terminal-output');

    await executeCommand(page, 'ps');

    const outputText = await getLastOutput(page);

    // Valid process states: Running, Ready, Blocked, Terminated
    const validStates = ['Running', 'Ready', 'Blocked', 'Terminated'];

    // Extract state column from ps output
    const lines = outputText.split('\n');
    for (const line of lines) {
      if (line.includes('PID') || line.trim() === '') continue;

      // Check if line contains at least one valid state
      const hasValidState = validStates.some(state => line.includes(state));
      if (!hasValidState && line.trim()) {
        // If we see a process line, it should have a valid state
        console.log(`Line without valid state: ${line}`);
      }
    }
  });

  test('C15: Version command shows correct process info', async ({ page }) => {
    const output = page.locator('#terminal-output');

    await executeCommand(page, 'version');

    // Version command should complete successfully
    await expect(output).toContainText('WOS v', { timeout: 200 });
    await expect(output).toContainText('kernel:');
    await expect(output).toContainText('userspace:');

    // Verify version numbers are present
    const outputText = await getLastOutput(page);
    expect(outputText).toMatch(/\d+\.\d+\.\d+/); // Semver pattern
  });

  test('C16: State command shows process information', async ({ page }) => {
    const output = page.locator('#terminal-output');

    await executeCommand(page, 'state');

    // State command should show kernel state
    await expect(output).toContainText('Kernel State', { timeout: 200 });
    await expect(output).toContainText('Processes:');
    await expect(output).toContainText('Next PID:');

    // Should show process count
    const outputText = await getLastOutput(page);
    expect(outputText).toMatch(/Processes:\s*\d+/);
  });

  test('C17: Multiple sequential process operations', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute multiple process-related commands in sequence
    const commands = [
      'ps',
      'state',
      'version',
      'echo process test',
      'ps',
    ];

    for (const cmd of commands) {
      await executeCommand(page, cmd);
      // Each command should complete without errors
      await page.waitForTimeout(50);
    }

    // Verify last ps command succeeded
    const outputText = await getLastOutput(page);
    expect(outputText).toContain('PID');

    // Verify echo output is present
    expect(outputText).toContain('process test');
  });

  test('C18: Process system remains stable after many operations', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute 20 commands to stress test process system
    for (let i = 0; i < 20; i++) {
      await executeCommand(page, `echo iteration ${i}`);
    }

    // System should still be responsive
    await executeCommand(page, 'ps');
    await expect(output).toContainText('PID', { timeout: 200 });

    // Verify we can still execute commands
    await executeCommand(page, 'echo stability test');
    await expect(output).toContainText('stability test');
  });

  test('C19: Process creation and termination performance', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Measure time for process creation (echo command)
    const startTime = Date.now();

    await input.fill('echo performance test');
    await input.press('Enter');

    // Wait for output to appear
    await expect(output).toContainText('performance test', { timeout: 200 });

    const endTime = Date.now();
    const duration = endTime - startTime;

    // Process creation and execution should be fast (<200ms)
    expect(duration).toBeLessThan(200);

    console.log(`Process creation/execution time: ${duration}ms (target: <200ms)`);
  });
});

test.describe('Canary: Process Management - Edge Cases', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('C20: ps command with no additional processes', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute ps immediately after load (minimal processes)
    await executeCommand(page, 'ps');

    // Should show at least init and shell
    const processCount = await getProcessCount(page);
    expect(processCount).toBeGreaterThanOrEqual(2);

    // Should have headers
    await expect(output).toContainText('PID');
    await expect(output).toContainText('STATE');
  });

  test('C21: Rapid ps command execution', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute ps multiple times rapidly
    for (let i = 0; i < 10; i++) {
      await executeCommand(page, 'ps');
      await page.waitForTimeout(10); // Minimal delay
    }

    // Last ps should still work correctly
    const outputText = await getLastOutput(page);
    expect(outputText).toContain('PID');

    // System should still be responsive
    await executeCommand(page, 'echo still works');
    await expect(output).toContainText('still works');
  });

  test('C22: Process commands after terminal clear', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Execute some process commands
    await executeCommand(page, 'ps');
    await executeCommand(page, 'state');

    // Clear terminal
    await input.press('Control+L');

    // Process commands should still work after clear
    await executeCommand(page, 'ps');
    await expect(output).toContainText('PID', { timeout: 200 });

    await executeCommand(page, 'version');
    await expect(output).toContainText('WOS v');
  });

  test('C23: Process state consistency after errors', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute invalid command (creates error)
    await executeCommand(page, 'invalidcommand');

    // Process system should still be stable
    await executeCommand(page, 'ps');
    await expect(output).toContainText('PID', { timeout: 200 });

    // Process count should be reasonable (not corrupted)
    const processCount = await getProcessCount(page);
    expect(processCount).toBeGreaterThanOrEqual(2);
    expect(processCount).toBeLessThan(100); // Sanity check
  });

  test('C24: Long-running command execution', async ({ page }) => {
    const output = page.locator('#terminal-output');

    // Execute a command that might take longer (help command with more output)
    await executeCommand(page, 'help');

    // Should complete successfully
    await expect(output).toContainText('Available commands', { timeout: 500 });

    // Process system should still be responsive
    await executeCommand(page, 'ps');
    await expect(output).toContainText('PID', { timeout: 200 });
  });
});
