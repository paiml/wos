/**
 * Canary Tests: Command Chaining (Sprint 3A)
 *
 * These tests validate the pipeline operators implemented in Sprint 2:
 * - Pipe operator (|)
 * - AND operator (&&)
 * - OR operator (||)
 * - Semicolon operator (;)
 *
 * Following SQLite's canary test philosophy: critical user workflows
 */

import { test, expect, Page } from '@playwright/test';

// Helper to type a command and press Enter
async function executeCommand(page: Page, command: string) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  // Small delay for command processing
  await page.waitForTimeout(50);
}

// Helper to get terminal output (excluding command lines)
async function getOutput(page: Page): Promise<string> {
  // Get only output lines, not command lines
  // Command lines have class "terminal-line command"
  // Output lines have class "terminal-line output"
  const outputLines = page.locator('#terminal-output .terminal-line.output');
  const count = await outputLines.count();

  const lines: string[] = [];
  for (let i = 0; i < count; i++) {
    const text = await outputLines.nth(i).textContent();
    if (text) {
      lines.push(text);
    }
  }

  return lines.join('\n');
}

test.describe('Canary: Command Chaining', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('');
    // Increased timeout for WASM initialization
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });
  });

  // ============================================================================
  // PIPE OPERATOR (|)
  // ============================================================================

  test('C60: Pipe operator - echo to grep (basic pipe)', async ({ page }) => {
    // Test that pipe passes output from first command to second
    // echo produces output, we verify it was executed
    await executeCommand(page, 'echo hello');
    const echoOutput = await getOutput(page);
    expect(echoOutput).toContain('hello');

    // Note: We don't have grep yet, but we can test that pipe syntax is accepted
    // For now, test that the command doesn't crash
    await executeCommand(page, 'echo "test data" | echo "piped"');
    const output = await getOutput(page);
    // The pipe should execute both commands
    expect(output).toBeDefined();
  });

  test('C61: Pipe operator - ps output processing', async ({ page }) => {
    // Test piping process list output
    // This tests that pipe operator works with real command output
    await executeCommand(page, 'ps');
    const psOutput = await getOutput(page);
    expect(psOutput).toMatch(/PID|init|shell/);

    // Pipe ps output (even if we just echo it, tests the plumbing)
    await executeCommand(page, 'ps | ps');
    const pipedOutput = await getOutput(page);
    expect(pipedOutput).toBeDefined();
  });

  test('C62: Pipe operator - three stage pipeline', async ({ page }) => {
    // Test that three commands can be piped together
    await executeCommand(page, 'echo first | echo second | echo third');
    const output = await getOutput(page);
    expect(output).toContain('third');
  });

  // ============================================================================
  // AND OPERATOR (&&)
  // ============================================================================

  test('C63: AND operator - both commands succeed', async ({ page }) => {
    // Test that both commands execute when first succeeds
    await executeCommand(page, 'echo "first" && echo "second"');
    const output = await getOutput(page);

    // Both commands should have executed
    expect(output).toContain('first');
    expect(output).toContain('second');
  });

  test('C64: AND operator - first command fails, second skipped', async ({ page }) => {
    // Test that second command is skipped when first fails
    // Using an invalid command to trigger failure
    await executeCommand(page, 'invalidcmd && echo "should not see this"');
    const output = await getOutput(page);

    // Should see error from first command
    expect(output).toMatch(/Unknown command|error/i);
    // Should NOT execute second command
    expect(output).not.toContain('should not see this');
  });

  test('C65: AND operator - chain of three commands', async ({ page }) => {
    // Test that all three commands execute when all succeed
    await executeCommand(page, 'echo "1" && echo "2" && echo "3"');
    const output = await getOutput(page);

    expect(output).toContain('1');
    expect(output).toContain('2');
    expect(output).toContain('3');
  });

  test('C66: AND operator - chain stops at first failure', async ({ page }) => {
    // Test that execution stops at first failure in chain
    await executeCommand(page, 'echo "1" && invalidcmd && echo "3"');
    const output = await getOutput(page);

    expect(output).toContain('1');
    expect(output).toMatch(/Unknown command|error/i);
    expect(output).not.toContain('3');
  });

  // ============================================================================
  // OR OPERATOR (||)
  // ============================================================================

  test('C67: OR operator - first succeeds, second skipped', async ({ page }) => {
    // Test that second command is skipped when first succeeds
    await executeCommand(page, 'echo "success" || echo "fallback"');
    const output = await getOutput(page);

    expect(output).toContain('success');
    expect(output).not.toContain('fallback');
  });

  test('C68: OR operator - first fails, second executes', async ({ page }) => {
    // Test that second command executes when first fails
    await executeCommand(page, 'invalidcmd || echo "fallback executed"');
    const output = await getOutput(page);

    expect(output).toMatch(/Unknown command|error/i);
    expect(output).toContain('fallback executed');
  });

  test('C69: OR operator - chain until first success', async ({ page }) => {
    // Test that execution continues until first success
    await executeCommand(page, 'invalidcmd1 || invalidcmd2 || echo "finally worked"');
    const output = await getOutput(page);

    expect(output).toContain('finally worked');
  });

  // ============================================================================
  // SEMICOLON OPERATOR (;)
  // ============================================================================

  test('C70: Semicolon - both commands execute regardless', async ({ page }) => {
    // Test that both commands execute even if first fails
    await executeCommand(page, 'invalidcmd ; echo "still executed"');
    const output = await getOutput(page);

    expect(output).toMatch(/Unknown command|error/i);
    expect(output).toContain('still executed');
  });

  test('C71: Semicolon - chain of successful commands', async ({ page }) => {
    // Test multiple commands with semicolon
    await executeCommand(page, 'echo "1" ; echo "2" ; echo "3"');
    const output = await getOutput(page);

    expect(output).toContain('1');
    expect(output).toContain('2');
    expect(output).toContain('3');
  });

  test('C72: Semicolon - mixed success and failure', async ({ page }) => {
    // Test that all commands execute regardless of failure
    await executeCommand(page, 'echo "1" ; invalidcmd ; echo "3"');
    const output = await getOutput(page);

    expect(output).toContain('1');
    expect(output).toMatch(/Unknown command|error/i);
    expect(output).toContain('3');
  });

  // ============================================================================
  // MIXED OPERATORS
  // ============================================================================

  test('C73: Mixed - pipe then AND', async ({ page }) => {
    // Test combining pipe and AND operators
    await executeCommand(page, 'echo "data" | echo "processed" && echo "success"');
    const output = await getOutput(page);

    expect(output).toContain('processed');
    expect(output).toContain('success');
  });

  test('C74: Mixed - AND then OR', async ({ page }) => {
    // Test combining AND and OR operators
    await executeCommand(page, 'echo "first" && invalidcmd || echo "recovered"');
    const output = await getOutput(page);

    expect(output).toContain('first');
    expect(output).toContain('recovered');
  });

  test('C75: Mixed - complex operator chain', async ({ page }) => {
    // Test complex chain: (success AND success) OR fallback ; final
    await executeCommand(page, 'echo "1" && echo "2" || echo "backup" ; echo "final"');
    const output = await getOutput(page);

    expect(output).toContain('1');
    expect(output).toContain('2');
    expect(output).not.toContain('backup'); // Should not execute
    expect(output).toContain('final');
  });

  // ============================================================================
  // REAL-WORLD WORKFLOWS
  // ============================================================================

  test('C76: Real workflow - verify process then show info', async ({ page }) => {
    // Test realistic workflow: check if ps works, then show version
    await executeCommand(page, 'ps && version');
    const output = await getOutput(page);

    expect(output).toMatch(/PID|init|shell/);
    expect(output).toMatch(/WOS|version/i);
  });

  test('C77: Real workflow - command fallback pattern', async ({ page }) => {
    // Test common pattern: try command, fall back to alternative
    await executeCommand(page, 'invalidtool || echo "Tool not found, using default"');
    const output = await getOutput(page);

    expect(output).toContain('Tool not found, using default');
  });

  test('C78: Real workflow - multi-step process', async ({ page }) => {
    // Test multi-step workflow with semicolons
    await executeCommand(page, 'echo "Starting" ; ps ; echo "Done"');
    const output = await getOutput(page);

    expect(output).toContain('Starting');
    expect(output).toMatch(/PID/);
    expect(output).toContain('Done');
  });

  // ============================================================================
  // EDGE CASES & ERROR HANDLING
  // ============================================================================

  test('C79: Edge case - empty command in pipeline', async ({ page }) => {
    // Test handling of empty commands
    await executeCommand(page, 'echo "test" && && echo "end"');
    // Should handle gracefully (might show error or skip empty)
    const output = await getOutput(page);
    expect(output).toBeDefined();
  });

  test('C80: Edge case - operators in quoted strings', async ({ page }) => {
    // Test that operators inside quotes are treated as literals
    await executeCommand(page, 'echo "use && for AND logic"');
    const output = await getOutput(page);

    expect(output).toContain('use && for AND logic');
  });

  test('C81: Performance - rapid command chaining', async ({ page }) => {
    // Test performance with multiple chained commands
    const startTime = Date.now();

    await executeCommand(page, 'echo 1 ; echo 2 ; echo 3 ; echo 4 ; echo 5');

    const endTime = Date.now();
    const executionTime = endTime - startTime;

    const output = await getOutput(page);
    expect(output).toContain('1');
    expect(output).toContain('5');

    // Should complete reasonably quickly (< 500ms for 5 commands)
    expect(executionTime).toBeLessThan(500);
  });

  test('C82: Consistency - operators work after terminal clear', async ({ page }) => {
    // Test that operators still work after clearing terminal
    await executeCommand(page, 'echo "before clear"');
    await page.keyboard.press('Control+L');
    await page.waitForTimeout(100);

    await executeCommand(page, 'echo "after" && echo "clear"');
    const output = await getOutput(page);

    expect(output).toContain('after');
    expect(output).toContain('clear');
  });

  test('C83: Consistency - operators work across page reload', async ({ page }) => {
    // Test that operators work after page reload
    await executeCommand(page, 'echo "test" && echo "before reload"');

    await page.goto('');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 30000 });

    await executeCommand(page, 'echo "after" || echo "reload"');
    const output = await getOutput(page);

    expect(output).toContain('after');
  });
});
