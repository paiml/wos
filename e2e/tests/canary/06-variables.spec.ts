/**
 * Canary Tests: Variables & Shell Parameters (Sprint 4)
 *
 * These tests validate shell variable functionality:
 * - Variable assignment (VAR=value)
 * - Variable expansion ($VAR, ${VAR})
 * - Exit status ($?)
 * - Export command
 *
 * Following SQLite's canary test philosophy: critical user workflows
 */

import { test, expect } from '@playwright/test';

// Helper to type a command and press Enter
async function executeCommand(page: any, command: string) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(50);
}

// Helper to get terminal output (excluding command lines)
async function getOutput(page: any): Promise<string> {
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

test.describe('Canary: Variables', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('');
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 5000 });
  });

  // ============================================================================
  // BASIC VARIABLE ASSIGNMENT
  // ============================================================================

  test('C84: Variable assignment - simple value', async ({ page }) => {
    // Set a variable
    await executeCommand(page, 'NAME=World');

    // Should not produce output (assignment is silent)
    const assignOutput = await getOutput(page);
    expect(assignOutput).not.toContain('NAME');

    // Use the variable
    await executeCommand(page, 'echo $NAME');
    const output = await getOutput(page);
    expect(output).toContain('World');
  });

  test('C85: Variable assignment - with spaces in quotes', async ({ page }) => {
    await executeCommand(page, 'GREETING="Hello World"');
    await executeCommand(page, 'echo $GREETING');
    const output = await getOutput(page);

    expect(output).toContain('Hello World');
  });

  test('C86: Variable assignment - empty value', async ({ page }) => {
    await executeCommand(page, 'EMPTY=');
    await executeCommand(page, 'echo "Value: $EMPTY end"');
    const output = await getOutput(page);

    expect(output).toContain('Value:  end');
  });

  test('C87: Variable assignment - numeric value', async ({ page }) => {
    await executeCommand(page, 'COUNT=42');
    await executeCommand(page, 'echo $COUNT');
    const output = await getOutput(page);

    expect(output).toContain('42');
  });

  // ============================================================================
  // VARIABLE EXPANSION
  // ============================================================================

  test('C88: Variable expansion - basic $VAR syntax', async ({ page }) => {
    await executeCommand(page, 'USER=alice');
    await executeCommand(page, 'echo $USER');
    const output = await getOutput(page);

    expect(output).toContain('alice');
  });

  test('C89: Variable expansion - ${VAR} syntax', async ({ page }) => {
    await executeCommand(page, 'FILE=test');
    await executeCommand(page, 'echo ${FILE}.txt');
    const output = await getOutput(page);

    expect(output).toContain('test.txt');
  });

  test('C90: Variable expansion - undefined variable', async ({ page }) => {
    // Undefined variables should expand to empty string
    await executeCommand(page, 'echo "Value: $UNDEFINED end"');
    const output = await getOutput(page);

    expect(output).toContain('Value:  end');
  });

  test('C91: Variable expansion - multiple variables', async ({ page }) => {
    await executeCommand(page, 'FIRST=John');
    await executeCommand(page, 'LAST=Doe');
    await executeCommand(page, 'echo $FIRST $LAST');
    const output = await getOutput(page);

    expect(output).toContain('John Doe');
  });

  test('C92: Variable expansion - in quoted string', async ({ page }) => {
    await executeCommand(page, 'NAME=Alice');
    await executeCommand(page, 'echo "Hello $NAME!"');
    const output = await getOutput(page);

    expect(output).toContain('Hello Alice!');
  });

  // ============================================================================
  // EXIT STATUS ($?)
  // ============================================================================

  test('C93: Exit status - successful command', async ({ page }) => {
    await executeCommand(page, 'echo hello');
    await executeCommand(page, 'echo $?');
    const output = await getOutput(page);

    expect(output).toContain('0');
  });

  test('C94: Exit status - failed command', async ({ page }) => {
    await executeCommand(page, 'invalidcommand');
    await executeCommand(page, 'echo $?');
    const output = await getOutput(page);

    expect(output).toContain('1');
  });

  test('C95: Exit status - chain of commands', async ({ page }) => {
    await executeCommand(page, 'echo first');
    await executeCommand(page, 'echo $?');
    const first = await getOutput(page);
    expect(first).toContain('0');

    await executeCommand(page, 'invalidcmd');
    await executeCommand(page, 'echo $?');
    const second = await getOutput(page);
    expect(second).toContain('1');
  });

  // ============================================================================
  // EXPORT COMMAND
  // ============================================================================

  test('C96: Export - basic export', async ({ page }) => {
    await executeCommand(page, 'export PATH=/usr/bin');
    await executeCommand(page, 'echo $PATH');
    const output = await getOutput(page);

    expect(output).toContain('/usr/bin');
  });

  test('C97: Export - without value (export existing var)', async ({ page }) => {
    await executeCommand(page, 'MYVAR=test');
    await executeCommand(page, 'export MYVAR');
    await executeCommand(page, 'echo $MYVAR');
    const output = await getOutput(page);

    expect(output).toContain('test');
  });

  test('C98: Export - multiple variables', async ({ page }) => {
    await executeCommand(page, 'export VAR1=one VAR2=two');
    await executeCommand(page, 'echo $VAR1 $VAR2');
    const output = await getOutput(page);

    expect(output).toContain('one two');
  });

  // ============================================================================
  // VARIABLE REASSIGNMENT
  // ============================================================================

  test('C99: Variable reassignment - overwrite value', async ({ page }) => {
    await executeCommand(page, 'X=first');
    await executeCommand(page, 'echo $X');
    let output = await getOutput(page);
    expect(output).toContain('first');

    await executeCommand(page, 'X=second');
    await executeCommand(page, 'echo $X');
    output = await getOutput(page);
    expect(output).toContain('second');
  });

  test('C100: Variable reassignment - type change', async ({ page }) => {
    await executeCommand(page, 'VAR=123');
    await executeCommand(page, 'echo $VAR');
    let output = await getOutput(page);
    expect(output).toContain('123');

    await executeCommand(page, 'VAR=text');
    await executeCommand(page, 'echo $VAR');
    output = await getOutput(page);
    expect(output).toContain('text');
  });

  // ============================================================================
  // COMPLEX SCENARIOS
  // ============================================================================

  test('C101: Variables with command chaining', async ({ page }) => {
    await executeCommand(page, 'VAR=test && echo $VAR || echo failed');
    const output = await getOutput(page);

    expect(output).toContain('test');
    expect(output).not.toContain('failed');
  });

  test('C102: Variables in pipeline', async ({ page }) => {
    await executeCommand(page, 'TEXT="hello world"');
    await executeCommand(page, 'echo $TEXT | grep hello');
    const output = await getOutput(page);

    expect(output).toContain('hello world');
  });

  test('C103: Variable persistence across commands', async ({ page }) => {
    // Set variable
    await executeCommand(page, 'SESSION=active');

    // Execute other commands
    await executeCommand(page, 'echo other command');
    await executeCommand(page, 'ps');

    // Variable should still exist
    await executeCommand(page, 'echo $SESSION');
    const output = await getOutput(page);
    expect(output).toContain('active');
  });

  // ============================================================================
  // EDGE CASES
  // ============================================================================

  test('C104: Edge case - variable name with underscore', async ({ page }) => {
    await executeCommand(page, 'MY_VAR=value');
    await executeCommand(page, 'echo $MY_VAR');
    const output = await getOutput(page);

    expect(output).toContain('value');
  });

  test('C105: Edge case - variable name with numbers', async ({ page }) => {
    await executeCommand(page, 'VAR123=value');
    await executeCommand(page, 'echo $VAR123');
    const output = await getOutput(page);

    expect(output).toContain('value');
  });

  test('C106: Edge case - dollar sign without variable', async ({ page }) => {
    // $ alone should be treated literally
    await executeCommand(page, 'echo "Price: $"');
    const output = await getOutput(page);

    expect(output).toContain('Price: $');
  });

  test('C107: Edge case - escaped dollar sign', async ({ page }) => {
    // \$ should not expand
    await executeCommand(page, 'VAR=test');
    await executeCommand(page, 'echo \\$VAR');
    const output = await getOutput(page);

    expect(output).toMatch(/\$VAR|\\$VAR/);
  });
});
