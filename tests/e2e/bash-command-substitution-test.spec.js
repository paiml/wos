// Bash Command Substitution Test (WOS-BASH-03)
// Reference: GNU Bash Manual - Command Substitution
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
  const text = await lastOutput.textContent();
  // Strip trailing newline (echo adds \n, but display should not show it)
  return text.replace(/\n$/, '');
}

async function getAllOutput(page) {
  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  let allOutput = '';
  for (let i = 0; i < count; i++) {
    const text = await outputs.nth(i).textContent();
    allOutput += text + '\n';
  }
  return allOutput.trim();
}

test.describe('Bash Command Substitution (WOS-BASH-03)', () => {
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

  // Basic command substitution
  test('$(cmd) captures command output', async ({ page }) => {
    await executeCommand(page, 'echo $(echo hello)');
    const output = await getLastOutput(page);
    expect(output).toBe('hello');
  });

  test('$(cmd) substitution in middle of string', async ({ page }) => {
    await executeCommand(page, 'echo prefix_$(echo middle)_suffix');
    const output = await getLastOutput(page);
    expect(output).toBe('prefix_middle_suffix');
  });

  test('$(cmd) with pwd command', async ({ page }) => {
    await executeCommand(page, 'echo Current dir is $(pwd)');
    const output = await getLastOutput(page);
    expect(output).toMatch(/Current dir is \//);
  });

  test('$(cmd) strips trailing newline', async ({ page }) => {
    await executeCommand(page, 'echo $(echo -e "hello\\n")world');
    const output = await getLastOutput(page);
    expect(output).toBe('helloworld');
  });

  // Multiple substitutions
  test('multiple $(cmd) in one command', async ({ page }) => {
    await executeCommand(page, 'echo $(echo first) and $(echo second)');
    const output = await getLastOutput(page);
    expect(output).toBe('first and second');
  });

  test('multiple $(cmd) with different commands', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/test.txt');
    await page.waitForTimeout(200);
    await executeCommand(page, 'echo pwd:$(pwd) files:$(ls /tmp/test.txt)');
    const output = await getLastOutput(page);
    expect(output).toContain('pwd:/');
    expect(output).toContain('files:/tmp/test.txt');
  });

  // Nested substitution
  test('nested $(cmd $(cmd)) single level', async ({ page }) => {
    await executeCommand(page, 'echo $(echo $(echo nested))');
    const output = await getLastOutput(page);
    expect(output).toBe('nested');
  });

  test('nested $(cmd $(cmd $(cmd))) multiple levels', async ({ page }) => {
    await executeCommand(page, 'echo $(echo $(echo $(echo deep)))');
    const output = await getLastOutput(page);
    expect(output).toBe('deep');
  });

  test('nested with different commands', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/file.txt');
    await page.waitForTimeout(200);
    await executeCommand(page, 'echo $(cat $(echo /tmp/file.txt))');
    const output = await getLastOutput(page);
    // File is empty, so output should be empty or just whitespace
    expect(output.trim()).toBe('');
  });

  // Variable expansion inside substitution
  test('variable expansion inside $(cmd)', async ({ page }) => {
    await executeCommand(page, 'NAME=world');
    await executeCommand(page, 'echo $(echo hello $NAME)');
    const output = await getLastOutput(page);
    expect(output).toBe('hello world');
  });

  test('$(cmd) result assigned to variable', async ({ page }) => {
    await executeCommand(page, 'RESULT=$(echo success)');
    await executeCommand(page, 'echo $RESULT');
    const output = await getLastOutput(page);
    expect(output).toBe('success');
  });

  test('$(cmd) in variable value with braces', async ({ page }) => {
    await executeCommand(page, 'PATH_VAL=$(pwd)');
    await executeCommand(page, 'echo Directory: ${PATH_VAL}');
    const output = await getLastOutput(page);
    expect(output).toMatch(/Directory: \//);
  });

  // Command substitution with pipes
  test('$(cmd | cmd) with pipe inside substitution', async ({ page }) => {
    await executeCommand(page, 'echo $(echo "hello world" | grep hello)');
    const output = await getLastOutput(page);
    expect(output).toBe('hello world');
  });

  test('$(cmd | cmd | cmd) multiple pipes', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/a.txt /tmp/b.txt');
    await page.waitForTimeout(200);
    await executeCommand(page, 'echo $(ls /tmp/*.txt | grep a.txt)');
    const output = await getLastOutput(page);
    expect(output).toContain('a.txt');
  });

  // Command substitution with quotes
  test('$(cmd) inside double quotes', async ({ page }) => {
    await executeCommand(page, 'echo "Result: $(echo test)"');
    const output = await getLastOutput(page);
    expect(output).toBe('Result: test');
  });

  test('$(cmd) with spaces preserved in quotes', async ({ page }) => {
    await executeCommand(page, 'echo "$(echo "hello  world")"');
    const output = await getLastOutput(page);
    expect(output).toBe('hello  world');
  });

  test('$(cmd) in single quotes is literal', async ({ page }) => {
    await executeCommand(page, "echo '$(echo test)'");
    const output = await getLastOutput(page);
    expect(output).toBe('$(echo test)');
  });

  // Edge cases
  test('empty $(cmd) produces empty string', async ({ page }) => {
    await executeCommand(page, 'echo start$(echo)end');
    const output = await getLastOutput(page);
    expect(output).toBe('startend');
  });

  test('$(cmd) with command that fails', async ({ page }) => {
    await executeCommand(page, 'RESULT=$(nonexistent_command)');
    await executeCommand(page, 'echo done');
    const output = await getLastOutput(page);
    // Should continue execution even if substitution command fails
    expect(output).toBe('done');
  });

  test('$(cmd) with multiline output becomes single line', async ({ page }) => {
    await executeCommand(page, 'echo "line1" > /tmp/multi.txt');
    await executeCommand(page, 'echo "line2" >> /tmp/multi.txt');
    await page.waitForTimeout(200);
    await executeCommand(page, 'echo $(cat /tmp/multi.txt)');
    const output = await getLastOutput(page);
    expect(output).toBe('line1 line2');
  });

  test('$(cmd) whitespace collapsed to single space', async ({ page }) => {
    await executeCommand(page, 'echo $(echo "a    b     c")');
    const output = await getLastOutput(page);
    expect(output).toBe('a b c');
  });

  // Complex real-world examples
  test('$(cmd) in command argument', async ({ page }) => {
    await executeCommand(page, 'echo "test content" > /tmp/file.txt');
    await page.waitForTimeout(200);
    await executeCommand(page, 'cat $(echo /tmp/file.txt)');
    const output = await getLastOutput(page);
    expect(output).toContain('test content');
  });

  test('$(cmd) with glob pattern result', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/x1.txt /tmp/x2.txt');
    await page.waitForTimeout(200);
    await executeCommand(page, 'PATTERN=$(echo "/tmp/x*.txt")');
    await executeCommand(page, 'ls $PATTERN');
    const output = await getAllOutput(page);
    expect(output).toContain('x1.txt');
    expect(output).toContain('x2.txt');
  });

  test('$(cmd) combining multiple command types', async ({ page }) => {
    await executeCommand(page, 'VAR=test');
    await executeCommand(page, 'echo ${VAR}_$(echo result)_$(pwd)');
    const output = await getLastOutput(page);
    expect(output).toMatch(/test_result_\//);
  });

  // Exit status handling
  test('$? reflects substitution command exit status', async ({ page }) => {
    await executeCommand(page, 'RESULT=$(echo success)');
    await executeCommand(page, 'echo $?');
    const output = await getLastOutput(page);
    expect(output).toBe('0');
  });

  test('$(false) sets exit status to 1', async ({ page }) => {
    await executeCommand(page, 'RESULT=$(false)');
    await executeCommand(page, 'echo $?');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  // Performance and complexity
  test('deeply nested substitutions work', async ({ page }) => {
    await executeCommand(page, 'echo $(echo $(echo $(echo $(echo five)))))');
    const output = await getLastOutput(page);
    expect(output).toBe('five');
  });

  test('$(cmd) with parameter expansion inside', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello');
    await executeCommand(page, 'echo $(echo ${TEXT^^})');
    const output = await getLastOutput(page);
    expect(output).toBe('HELLO');
  });
});
