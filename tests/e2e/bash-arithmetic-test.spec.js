// Bash Arithmetic Expansion Test (WOS-BASH-09)
// Reference: GNU Bash Manual - Arithmetic Expansion
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

test.describe('Bash Arithmetic Expansion (WOS-BASH-09)', () => {
  test.beforeEach(async ({ page }) => {
    // Cache-bust the HTML page itself
    const cacheBuster = Date.now();
    await page.goto(`http://127.0.0.1:8000/?v=${cacheBuster}`);
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForTimeout(2000);

    const skipButton = page.locator('button:has-text("Skip Tutorial")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForTimeout(500);
    }
  });

  // Basic arithmetic operations
  test('addition', async ({ page }) => {
    await executeCommand(page, 'echo $((2 + 3))');
    const output = await getLastOutput(page);
    expect(output).toBe('5');
  });

  test('subtraction', async ({ page }) => {
    await executeCommand(page, 'echo $((10 - 4))');
    const output = await getLastOutput(page);
    expect(output).toBe('6');
  });

  test('multiplication', async ({ page }) => {
    await executeCommand(page, 'echo $((6 * 7))');
    const output = await getLastOutput(page);
    expect(output).toBe('42');
  });

  test('division', async ({ page }) => {
    await executeCommand(page, 'echo $((20 / 5))');
    const output = await getLastOutput(page);
    expect(output).toBe('4');
  });

  test('modulo', async ({ page }) => {
    await executeCommand(page, 'echo $((17 % 5))');
    const output = await getLastOutput(page);
    expect(output).toBe('2');
  });

  test('integer division truncates', async ({ page }) => {
    await executeCommand(page, 'echo $((7 / 2))');
    const output = await getLastOutput(page);
    expect(output).toBe('3'); // Integer division
  });

  // Operator precedence
  test('multiplication before addition', async ({ page }) => {
    await executeCommand(page, 'echo $((2 + 3 * 4))');
    const output = await getLastOutput(page);
    expect(output).toBe('14'); // 2 + 12 = 14
  });

  test('parentheses override precedence', async ({ page }) => {
    await executeCommand(page, 'echo $(((2 + 3) * 4))');
    const output = await getLastOutput(page);
    expect(output).toBe('20'); // 5 * 4 = 20
  });

  test('complex expression with precedence', async ({ page }) => {
    await executeCommand(page, 'echo $((2 + 3 * 4 - 5))');
    const output = await getLastOutput(page);
    expect(output).toBe('9'); // 2 + 12 - 5 = 9
  });

  // Negative numbers
  test('negative number literal', async ({ page }) => {
    await executeCommand(page, 'echo $((-5))');
    const output = await getLastOutput(page);
    expect(output).toBe('-5');
  });

  test('subtraction resulting in negative', async ({ page }) => {
    await executeCommand(page, 'echo $((3 - 10))');
    const output = await getLastOutput(page);
    expect(output).toBe('-7');
  });

  test('multiplication with negative', async ({ page }) => {
    await executeCommand(page, 'echo $((-3 * 4))');
    const output = await getLastOutput(page);
    expect(output).toBe('-12');
  });

  // Variables in arithmetic
  test('variable expansion in arithmetic', async ({ page }) => {
    await executeCommand(page, 'NUM=42');
    await executeCommand(page, 'echo $((NUM + 8))');
    const output = await getLastOutput(page);
    expect(output).toBe('50');
  });

  test('multiple variables in expression', async ({ page }) => {
    await executeCommand(page, 'A=10');
    await executeCommand(page, 'B=20');
    await executeCommand(page, 'echo $((A + B))');
    const output = await getLastOutput(page);
    expect(output).toBe('30');
  });

  test('variable without dollar sign in arithmetic', async ({ page }) => {
    await executeCommand(page, 'X=7');
    await executeCommand(page, 'echo $((X * 3))');
    const output = await getLastOutput(page);
    expect(output).toBe('21'); // Bash allows X without $
  });

  test('undefined variable treated as zero', async ({ page }) => {
    await executeCommand(page, 'echo $((UNDEFINED + 5))');
    const output = await getLastOutput(page);
    expect(output).toBe('5'); // Undefined = 0
  });

  // Comparison operators
  test('less than returns 1 if true', async ({ page }) => {
    await executeCommand(page, 'echo $((3 < 5))');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  test('less than returns 0 if false', async ({ page }) => {
    await executeCommand(page, 'echo $((5 < 3))');
    const output = await getLastOutput(page);
    expect(output).toBe('0');
  });

  test('greater than', async ({ page }) => {
    await executeCommand(page, 'echo $((10 > 5))');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  test('less than or equal', async ({ page }) => {
    await executeCommand(page, 'echo $((5 <= 5))');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  test('greater than or equal', async ({ page }) => {
    await executeCommand(page, 'echo $((10 >= 20))');
    const output = await getLastOutput(page);
    expect(output).toBe('0');
  });

  test('equality comparison', async ({ page }) => {
    await executeCommand(page, 'echo $((7 == 7))');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  test('inequality comparison', async ({ page }) => {
    await executeCommand(page, 'echo $((5 != 3))');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  // Logical operators
  test('logical AND with both true', async ({ page }) => {
    await executeCommand(page, 'echo $((1 && 1))');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  test('logical AND with one false', async ({ page }) => {
    await executeCommand(page, 'echo $((1 && 0))');
    const output = await getLastOutput(page);
    expect(output).toBe('0');
  });

  test('logical OR with one true', async ({ page }) => {
    await executeCommand(page, 'echo $((0 || 1))');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  test('logical OR with both false', async ({ page }) => {
    await executeCommand(page, 'echo $((0 || 0))');
    const output = await getLastOutput(page);
    expect(output).toBe('0');
  });

  test('logical NOT', async ({ page }) => {
    await executeCommand(page, 'echo $((!0))');
    const output = await getLastOutput(page);
    expect(output).toBe('1');
  });

  test('logical NOT of non-zero', async ({ page }) => {
    await executeCommand(page, 'echo $((!5))');
    const output = await getLastOutput(page);
    expect(output).toBe('0');
  });

  // Bitwise operators
  test('bitwise AND', async ({ page }) => {
    await executeCommand(page, 'echo $((12 & 10))');
    const output = await getLastOutput(page);
    expect(output).toBe('8'); // 1100 & 1010 = 1000
  });

  test('bitwise OR', async ({ page }) => {
    await executeCommand(page, 'echo $((12 | 10))');
    const output = await getLastOutput(page);
    expect(output).toBe('14'); // 1100 | 1010 = 1110
  });

  test('bitwise XOR', async ({ page }) => {
    await executeCommand(page, 'echo $((12 ^ 10))');
    const output = await getLastOutput(page);
    expect(output).toBe('6'); // 1100 ^ 1010 = 0110
  });

  test('bitwise NOT', async ({ page }) => {
    await executeCommand(page, 'echo $((~5))');
    const output = await getLastOutput(page);
    expect(output).toBe('-6'); // Two's complement
  });

  test('left shift', async ({ page }) => {
    await executeCommand(page, 'echo $((3 << 2))');
    const output = await getLastOutput(page);
    expect(output).toBe('12'); // 3 * 4 = 12
  });

  test('right shift', async ({ page }) => {
    await executeCommand(page, 'echo $((12 >> 2))');
    const output = await getLastOutput(page);
    expect(output).toBe('3'); // 12 / 4 = 3
  });

  // Whitespace handling
  test('arithmetic with no spaces', async ({ page }) => {
    await executeCommand(page, 'echo $((5+3))');
    const output = await getLastOutput(page);
    expect(output).toBe('8');
  });

  test('arithmetic with extra spaces', async ({ page }) => {
    await executeCommand(page, 'echo $((  5  +  3  ))');
    const output = await getLastOutput(page);
    expect(output).toBe('8');
  });

  // Nested arithmetic
  test('arithmetic in string context', async ({ page }) => {
    await executeCommand(page, 'echo Result: $((10 + 20))');
    const output = await getLastOutput(page);
    expect(output).toBe('Result: 30');
  });

  test('multiple arithmetic expansions', async ({ page }) => {
    await executeCommand(page, 'echo $((2 + 2)) plus $((3 + 3))');
    const output = await getLastOutput(page);
    expect(output).toBe('4 plus 6');
  });

  test('arithmetic in variable assignment', async ({ page }) => {
    await executeCommand(page, 'RESULT=$((5 * 6))');
    await executeCommand(page, 'echo $RESULT');
    const output = await getLastOutput(page);
    expect(output).toBe('30');
  });

  // Ternary operator
  test('ternary operator true case', async ({ page }) => {
    await executeCommand(page, 'echo $((1 ? 42 : 99))');
    const output = await getLastOutput(page);
    expect(output).toBe('42');
  });

  test('ternary operator false case', async ({ page }) => {
    await executeCommand(page, 'echo $((0 ? 42 : 99))');
    const output = await getLastOutput(page);
    expect(output).toBe('99');
  });

  // Edge cases
  test('division by zero returns error', async ({ page }) => {
    await executeCommand(page, 'echo $((10 / 0))');
    const output = await getLastOutput(page);
    // Should contain error message (implementation dependent)
    expect(output).toContain('division by zero');
  });

  test('empty arithmetic expansion', async ({ page }) => {
    await executeCommand(page, 'echo $(())');
    const output = await getLastOutput(page);
    expect(output).toBe('0'); // Empty expression = 0
  });

  test('arithmetic with only spaces', async ({ page }) => {
    await executeCommand(page, 'echo $((   ))');
    const output = await getLastOutput(page);
    expect(output).toBe('0');
  });

  // Complex real-world examples
  test('calculate percentage', async ({ page }) => {
    await executeCommand(page, 'TOTAL=200');
    await executeCommand(page, 'PART=50');
    await executeCommand(page, 'echo $((PART * 100 / TOTAL))');
    const output = await getLastOutput(page);
    expect(output).toBe('25');
  });

  test('increment pattern', async ({ page }) => {
    await executeCommand(page, 'COUNT=5');
    await executeCommand(page, 'echo $((COUNT + 1))');
    const output = await getLastOutput(page);
    expect(output).toBe('6');
  });

  test('arithmetic in command substitution', async ({ page }) => {
    await executeCommand(page, 'echo $(echo $((3 + 4)))');
    const output = await getLastOutput(page);
    expect(output).toBe('7');
  });
});
