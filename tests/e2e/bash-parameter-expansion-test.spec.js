// Bash Parameter Expansion Test (WOS-BASH-05)
// Reference: GNU Bash Manual - Shell Parameter Expansion
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

test.describe('Bash Parameter Expansion (WOS-BASH-05)', () => {
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

  // Default value expansion
  test('${var:-default} uses default when unset', async ({ page }) => {
    await executeCommand(page, 'echo ${UNSET_VAR:-default_value}');
    const output = await getLastOutput(page);
    console.log(`Default value output: "${output}"`);
    expect(output).toContain('default_value');
  });

  test('${var:-default} uses variable value when set', async ({ page }) => {
    await executeCommand(page, 'TEST=actual');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEST:-default}');
    const output = await getLastOutput(page);
    expect(output).toContain('actual');
    expect(output).not.toContain('default');
  });

  test('${var:-default} uses default when empty', async ({ page }) => {
    await executeCommand(page, 'EMPTY=');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${EMPTY:-default}');
    const output = await getLastOutput(page);
    expect(output).toContain('default');
  });

  // Assign default value
  test('${var:=default} assigns and returns default when unset', async ({ page }) => {
    await executeCommand(page, 'echo ${NEW_VAR:=assigned}');
    const output1 = await getLastOutput(page);
    expect(output1).toContain('assigned');

    // Verify variable was actually set
    await executeCommand(page, 'echo $NEW_VAR');
    const output2 = await getLastOutput(page);
    expect(output2).toContain('assigned');
  });

  test('${var:=default} uses existing value when set', async ({ page }) => {
    await executeCommand(page, 'EXISTING=original');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${EXISTING:=new}');
    const output = await getLastOutput(page);
    expect(output).toContain('original');
    expect(output).not.toContain('new');
  });

  // Error if unset
  test('${var:?error} displays error when unset', async ({ page }) => {
    await executeCommand(page, 'echo ${MISSING:?variable is required}');
    const output = await getLastOutput(page);
    console.log(`Error output: "${output}"`);
    expect(output).toMatch(/error|required|MISSING/i);
  });

  test('${var:?error} returns value when set', async ({ page }) => {
    await executeCommand(page, 'PRESENT=value');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${PRESENT:?error}');
    const output = await getLastOutput(page);
    expect(output).toContain('value');
    expect(output).not.toMatch(/error/i);
  });

  // Use alternate value
  test('${var:+alternate} returns alternate when set', async ({ page }) => {
    await executeCommand(page, 'SET_VAR=something');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${SET_VAR:+alternate}');
    const output = await getLastOutput(page);
    expect(output).toContain('alternate');
    expect(output).not.toContain('something');
  });

  test('${var:+alternate} returns empty when unset', async ({ page }) => {
    await executeCommand(page, 'echo ${UNSET:+alternate}');
    const output = await getLastOutput(page);
    expect(output.trim()).toBe('');
  });

  // String length
  test('${#var} returns string length', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${#TEXT}');
    const output = await getLastOutput(page);
    console.log(`Length output: "${output}"`);
    expect(output).toContain('5');
  });

  test('${#var} returns 0 for empty string', async ({ page }) => {
    await executeCommand(page, 'EMPTY=');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${#EMPTY}');
    const output = await getLastOutput(page);
    expect(output).toContain('0');
  });

  test('${#var} returns 0 for unset variable', async ({ page }) => {
    await executeCommand(page, 'echo ${#UNSET}');
    const output = await getLastOutput(page);
    expect(output).toContain('0');
  });

  // Substring expansion
  test('${var:offset} extracts substring from offset', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello_world');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT:6}');
    const output = await getLastOutput(page);
    expect(output).toContain('world');
  });

  test('${var:offset:length} extracts substring with length', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello_world');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT:0:5}');
    const output = await getLastOutput(page);
    expect(output).toContain('hello');
  });

  test('${var:offset} with negative offset counts from end', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello_world');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT: -5}'); // Note: space before minus
    const output = await getLastOutput(page);
    expect(output).toContain('world');
  });

  // Pattern removal - remove shortest prefix
  test('${var#pattern} removes shortest prefix match', async ({ page }) => {
    await executeCommand(page, 'PATH=/usr/local/bin');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${PATH#*/}');
    const output = await getLastOutput(page);
    expect(output).toContain('usr/local/bin');
    expect(output).not.toMatch(/^\//);
  });

  test('${var##pattern} removes longest prefix match', async ({ page }) => {
    await executeCommand(page, 'PATH=/usr/local/bin');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${PATH##*/}');
    const output = await getLastOutput(page);
    expect(output).toContain('bin');
    expect(output).not.toContain('/');
  });

  // Pattern removal - remove shortest suffix
  test('${var%pattern} removes shortest suffix match', async ({ page }) => {
    await executeCommand(page, 'FILE=document.txt.bak');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${FILE%.*}');
    const output = await getLastOutput(page);
    expect(output).toContain('document.txt');
    expect(output).not.toContain('.bak');
  });

  test('${var%%pattern} removes longest suffix match', async ({ page }) => {
    await executeCommand(page, 'FILE=document.txt.bak');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${FILE%%.*}');
    const output = await getLastOutput(page);
    expect(output).toContain('document');
    expect(output).not.toContain('.');
  });

  // Case modification
  test('${var^} capitalizes first character', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT^}');
    const output = await getLastOutput(page);
    expect(output).toContain('Hello');
  });

  test('${var^^} converts to uppercase', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT^^}');
    const output = await getLastOutput(page);
    expect(output).toContain('HELLO');
  });

  test('${var,} lowercases first character', async ({ page }) => {
    await executeCommand(page, 'TEXT=HELLO');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT,}');
    const output = await getLastOutput(page);
    expect(output).toContain('hELLO');
  });

  test('${var,,} converts to lowercase', async ({ page }) => {
    await executeCommand(page, 'TEXT=HELLO');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT,,}');
    const output = await getLastOutput(page);
    expect(output).toContain('hello');
  });

  // Pattern substitution
  test('${var/pattern/replacement} replaces first match', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello_hello_world');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT/hello/goodbye}');
    const output = await getLastOutput(page);
    expect(output).toContain('goodbye_hello_world');
  });

  test('${var//pattern/replacement} replaces all matches', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello_hello_world');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT//hello/goodbye}');
    const output = await getLastOutput(page);
    expect(output).toContain('goodbye_goodbye_world');
    expect(output).not.toContain('hello');
  });

  test('${var/#pattern/replacement} replaces at beginning', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello_world');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT/#hello/goodbye}');
    const output = await getLastOutput(page);
    expect(output).toContain('goodbye_world');
  });

  test('${var/%pattern/replacement} replaces at end', async ({ page }) => {
    await executeCommand(page, 'TEXT=hello_world');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${TEXT/%world/universe}');
    const output = await getLastOutput(page);
    expect(output).toContain('hello_universe');
  });

  // Combined operations
  test('nested parameter expansions', async ({ page }) => {
    await executeCommand(page, 'OUTER=inner');
    await executeCommand(page, 'inner=value');
    await page.waitForTimeout(100);
    // This would be ${!OUTER} for indirect expansion, but testing nested defaults
    await executeCommand(page, 'echo ${OUTER:-${inner:-default}}');
    const output = await getLastOutput(page);
    expect(output).toContain('inner');
  });

  test('parameter expansion with special characters', async ({ page }) => {
    await executeCommand(page, 'VAR=hello-world');
    await page.waitForTimeout(100);
    await executeCommand(page, 'echo ${VAR//-/_}');
    const output = await getLastOutput(page);
    expect(output).toContain('hello_world');
  });
});
