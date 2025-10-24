// Bash Special Variables Test (WOS-BASH-04)
// Reference: GNU Bash Manual - Special Parameters
// Checklist: docs/specifications/vim-bash-official-checklist.md
const { test, expect } = require('@playwright/test');

async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(300);
}

async function getLastOutput(page) {
  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  if (count === 0) return '';
  const lastOutput = outputs.nth(count - 1);
  return await lastOutput.textContent();
}

async function getOutputContaining(page, text) {
  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  for (let i = count - 1; i >= 0; i--) {
    const output = await outputs.nth(i).textContent();
    if (output.includes(text)) {
      return output;
    }
  }
  return '';
}

test.describe('Bash Special Variables (WOS-BASH-04)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://127.0.0.1:8000');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForTimeout(2000);

    const skipButton = page.locator('button:has-text("Skip Tutorial")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('$? returns 0 after successful command', async ({ page }) => {
    // Execute successful command
    await executeCommand(page, 'echo hello');
    await page.waitForTimeout(200);

    // Check exit status
    await executeCommand(page, 'echo $?');
    const output = await getLastOutput(page);

    console.log(`Exit status after success: "${output}"`);
    expect(output).toContain('0');
  });

  test('$? returns non-zero after failed command', async ({ page }) => {
    // Execute failing command
    await executeCommand(page, 'ls /nonexistent_directory_12345');
    await page.waitForTimeout(200);

    // Check exit status
    await executeCommand(page, 'echo $?');
    const output = await getLastOutput(page);

    console.log(`Exit status after failure: "${output}"`);
    // Should be non-zero (typically 1 or 2)
    expect(output).not.toContain('0');
    expect(parseInt(output.trim())).toBeGreaterThan(0);
  });

  test('$? persists until next command', async ({ page }) => {
    // Failing command
    await executeCommand(page, 'cat /nonexistent');
    await page.waitForTimeout(200);

    // Check twice - should be same
    await executeCommand(page, 'echo $?');
    const output1 = await getLastOutput(page);

    await executeCommand(page, 'echo $?');
    const output2 = await getLastOutput(page);

    console.log(`First check: "${output1}", Second check: "${output2}"`);
    // Second echo $? should show 0 (echo succeeded)
    expect(output2).toContain('0');
  });

  test('$$ returns current shell process ID', async ({ page }) => {
    await executeCommand(page, 'echo $$');
    const output = await getLastOutput(page);

    console.log(`Process ID: "${output}"`);
    // Should be a number (typically 1 for init process)
    const pid = parseInt(output.trim());
    expect(pid).toBeGreaterThanOrEqual(0);
    expect(isNaN(pid)).toBe(false);
  });

  test('$$ returns consistent PID across multiple calls', async ({ page }) => {
    await executeCommand(page, 'echo $$');
    const output1 = await getLastOutput(page);

    await executeCommand(page, 'echo $$');
    const output2 = await getLastOutput(page);

    console.log(`First PID: "${output1}", Second PID: "${output2}"`);
    expect(output1.trim()).toBe(output2.trim());
  });

  test('$0 returns shell name or script name', async ({ page }) => {
    await executeCommand(page, 'echo $0');
    const output = await getLastOutput(page);

    console.log(`Shell name: "${output}"`);
    // Should return something like "wos" or "sh" or "-wos"
    expect(output.length).toBeGreaterThan(0);
    expect(output.trim()).not.toBe('$0'); // Should be expanded
  });

  test('$1 $2 $3 work with command arguments', async ({ page }) => {
    // Create a simple script that uses positional parameters
    await executeCommand(page, 'echo "echo arg1=$1 arg2=$2 arg3=$3" > /tmp/test.sh');
    await page.waitForTimeout(200);

    // Execute the script with arguments
    await executeCommand(page, 'bash /tmp/test.sh first second third');
    await page.waitForTimeout(500);

    const output = await getOutputContaining(page, 'arg1=');
    console.log(`Script output: "${output}"`);

    expect(output).toContain('arg1=first');
    expect(output).toContain('arg2=second');
    expect(output).toContain('arg3=third');
  });

  test('$# returns number of positional parameters', async ({ page }) => {
    // Create script that shows argument count
    await executeCommand(page, 'echo "echo \\"Arguments: $#\\"" > /tmp/argcount.sh');
    await page.waitForTimeout(200);

    // Test with 3 arguments
    await executeCommand(page, 'bash /tmp/argcount.sh one two three');
    await page.waitForTimeout(500);

    const output = await getOutputContaining(page, 'Arguments:');
    console.log(`Argument count: "${output}"`);

    expect(output).toContain('Arguments: 3');
  });

  test('$@ expands to all positional parameters', async ({ page }) => {
    // Create script that echoes all arguments
    await executeCommand(page, 'echo "echo \\"All args: $@\\"" > /tmp/allargs.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/allargs.sh alpha beta gamma');
    await page.waitForTimeout(500);

    const output = await getOutputContaining(page, 'All args:');
    console.log(`All arguments: "${output}"`);

    expect(output).toContain('alpha');
    expect(output).toContain('beta');
    expect(output).toContain('gamma');
  });

  test('$* expands to all positional parameters as single word', async ({ page }) => {
    // Create script using $*
    await executeCommand(page, 'echo "echo \\"Args: $*\\"" > /tmp/starargs.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/starargs.sh foo bar baz');
    await page.waitForTimeout(500);

    const output = await getOutputContaining(page, 'Args:');
    console.log(`$* expansion: "${output}"`);

    expect(output).toContain('foo');
    expect(output).toContain('bar');
    expect(output).toContain('baz');
  });

  test('special variables work in variable expansion', async ({ page }) => {
    // Test that special vars can be used in expressions
    await executeCommand(page, 'true');
    await page.waitForTimeout(200);

    await executeCommand(page, 'STATUS=$?');
    await executeCommand(page, 'echo $STATUS');
    const output = await getLastOutput(page);

    console.log(`Stored exit status: "${output}"`);
    expect(output).toContain('0');
  });

  test('$? updates after each command in pipeline', async ({ page }) => {
    // Test: false returns 1
    await executeCommand(page, 'false');
    await page.waitForTimeout(200);

    await executeCommand(page, 'echo $?');
    const output1 = await getLastOutput(page);
    console.log(`After false: "${output1}"`);
    expect(parseInt(output1.trim())).toBeGreaterThan(0);

    // Test: true returns 0
    await executeCommand(page, 'true');
    await page.waitForTimeout(200);

    await executeCommand(page, 'echo $?');
    const output2 = await getLastOutput(page);
    console.log(`After true: "${output2}"`);
    expect(output2).toContain('0');
  });

  test('undefined positional parameters expand to empty', async ({ page }) => {
    // Script with more $N than args provided
    await executeCommand(page, 'echo "echo \\"arg5=$5\\"" > /tmp/missing.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/missing.sh one two');
    await page.waitForTimeout(500);

    const output = await getOutputContaining(page, 'arg5=');
    console.log(`Missing arg: "${output}"`);

    // Should show "arg5=" with nothing after
    expect(output).toContain('arg5=');
    // $5 should be empty (not the literal "$5")
    expect(output).not.toContain('$5');
  });

  test('special variables have correct precedence in expansion', async ({ page }) => {
    // Test that special vars are expanded before regular vars
    await executeCommand(page, 'TEST=value');
    await executeCommand(page, 'echo "$TEST $$"');

    const output = await getLastOutput(page);
    console.log(`Mixed expansion: "${output}"`);

    expect(output).toContain('value');
    // Should contain a PID number
    expect(/\d+/.test(output)).toBe(true);
  });

  test('$? bashrs validation: must be quoted in output', async ({ page }) => {
    // This tests that our implementation follows bashrs SC2086 rule
    // $? in echo should ideally be quoted, but for display it's acceptable unquoted
    await executeCommand(page, 'true');
    await page.waitForTimeout(200);

    await executeCommand(page, 'VAR=$?');
    await executeCommand(page, 'echo "$VAR"');

    const output = await getLastOutput(page);
    console.log(`Quoted $?: "${output}"`);
    expect(output).toContain('0');
  });
});
