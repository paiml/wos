// Bash Control Structures Test (WOS-BASH-06)
// Reference: GNU Bash Manual - Conditional Constructs & Looping Constructs
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

test.describe('Bash Control Structures (WOS-BASH-06)', () => {
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

  // IF statement tests
  test('if-then-fi basic structure', async ({ page }) => {
    // Create script with if statement
    await executeCommand(page, 'echo "#!/bin/bash\nif true; then\n  echo success\nfi" > /tmp/if_test.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/if_test.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    console.log(`if-then-fi output: "${output}"`);
    expect(output).toContain('success');
  });

  test('if-then-else structure', async ({ page }) => {
    // Test true condition
    await executeCommand(page, 'echo "#!/bin/bash\nif true; then\n  echo true-branch\nelse\n  echo false-branch\nfi" > /tmp/if_else.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/if_else.sh');
    await page.waitForTimeout(500);

    const output1 = await getLastOutput(page);
    expect(output1).toContain('true-branch');
    expect(output1).not.toContain('false-branch');

    // Test false condition
    await executeCommand(page, 'echo "#!/bin/bash\nif false; then\n  echo true-branch\nelse\n  echo false-branch\nfi" > /tmp/if_else2.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/if_else2.sh');
    await page.waitForTimeout(500);

    const output2 = await getLastOutput(page);
    expect(output2).toContain('false-branch');
    expect(output2).not.toContain('true-branch');
  });

  test('if-elif-else structure', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nVAL=2\nif [ $VAL -eq 1 ]; then\n  echo one\nelif [ $VAL -eq 2 ]; then\n  echo two\nelse\n  echo other\nfi" > /tmp/if_elif.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/if_elif.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    console.log(`if-elif-else output: "${output}"`);
    expect(output).toContain('two');
  });

  test('nested if statements', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nif true; then\n  if true; then\n    echo nested-success\n  fi\nfi" > /tmp/nested_if.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/nested_if.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('nested-success');
  });

  // WHILE loop tests
  test('while loop basic structure', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nCOUNT=0\nwhile [ $COUNT -lt 3 ]; do\n  echo $COUNT\n  COUNT=$((COUNT + 1))\ndone" > /tmp/while_test.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/while_test.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    console.log(`while loop output: "${output}"`);
    expect(output).toContain('0');
    expect(output).toContain('1');
    expect(output).toContain('2');
    expect(output).not.toContain('3');
  });

  test('while loop with break', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nCOUNT=0\nwhile true; do\n  echo $COUNT\n  COUNT=$((COUNT + 1))\n  if [ $COUNT -ge 2 ]; then\n    break\n  fi\ndone" > /tmp/while_break.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/while_break.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    expect(output).toContain('0');
    expect(output).toContain('1');
    expect(output).not.toContain('2');
  });

  test('while loop with continue', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nCOUNT=0\nwhile [ $COUNT -lt 3 ]; do\n  COUNT=$((COUNT + 1))\n  if [ $COUNT -eq 2 ]; then\n    continue\n  fi\n  echo $COUNT\ndone" > /tmp/while_continue.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/while_continue.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    expect(output).toContain('1');
    expect(output).not.toContain('2'); // Skipped by continue
    expect(output).toContain('3');
  });

  // FOR loop tests
  test('for loop with list', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nfor item in one two three; do\n  echo $item\ndone" > /tmp/for_list.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/for_list.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    console.log(`for loop output: "${output}"`);
    expect(output).toContain('one');
    expect(output).toContain('two');
    expect(output).toContain('three');
  });

  test('for loop with numeric range', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nfor i in 1 2 3; do\n  echo $i\ndone" > /tmp/for_range.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/for_range.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    expect(output).toContain('1');
    expect(output).toContain('2');
    expect(output).toContain('3');
  });

  test('for loop with variable expansion', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nFILES=\\\"file1 file2 file3\\\"\nfor f in $FILES; do\n  echo $f\ndone" > /tmp/for_var.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/for_var.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    expect(output).toContain('file1');
    expect(output).toContain('file2');
    expect(output).toContain('file3');
  });

  test('for loop with break', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nfor i in 1 2 3 4 5; do\n  echo $i\n  if [ $i -eq 3 ]; then\n    break\n  fi\ndone" > /tmp/for_break.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/for_break.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    expect(output).toContain('1');
    expect(output).toContain('2');
    expect(output).toContain('3');
    expect(output).not.toContain('4');
  });

  // CASE statement tests
  test('case statement basic structure', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nVAL=two\ncase $VAL in\n  one) echo 1 ;;\n  two) echo 2 ;;\n  three) echo 3 ;;\nesac" > /tmp/case_basic.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/case_basic.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    console.log(`case statement output: "${output}"`);
    expect(output).toContain('2');
  });

  test('case statement with default pattern', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nVAL=unknown\ncase $VAL in\n  one) echo 1 ;;\n  two) echo 2 ;;\n  *) echo default ;;\nesac" > /tmp/case_default.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/case_default.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('default');
  });

  test('case statement with pattern alternatives', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nVAL=y\ncase $VAL in\n  y|yes) echo affirmative ;;\n  n|no) echo negative ;;\nesac" > /tmp/case_alt.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/case_alt.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('affirmative');

    // Test second alternative
    await executeCommand(page, 'echo "#!/bin/bash\nVAL=yes\ncase $VAL in\n  y|yes) echo affirmative ;;\n  n|no) echo negative ;;\nesac" > /tmp/case_alt2.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/case_alt2.sh');
    await page.waitForTimeout(500);

    const output2 = await getLastOutput(page);
    expect(output2).toContain('affirmative');
  });

  // Test expression tests
  test('test command with -eq numeric comparison', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nif [ 5 -eq 5 ]; then\n  echo equal\nfi" > /tmp/test_eq.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_eq.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('equal');
  });

  test('test command with -lt numeric comparison', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nif [ 3 -lt 5 ]; then\n  echo less\nfi" > /tmp/test_lt.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_lt.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('less');
  });

  test('test command with -gt numeric comparison', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nif [ 7 -gt 5 ]; then\n  echo greater\nfi" > /tmp/test_gt.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_gt.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('greater');
  });

  test('test command with string comparison', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nif [ \\\"hello\\\" = \\\"hello\\\" ]; then\n  echo match\nfi" > /tmp/test_str.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_str.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('match');
  });

  test('test command with -z (zero length string)', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nEMPTY=\nif [ -z \\\"$EMPTY\\\" ]; then\n  echo empty\nfi" > /tmp/test_z.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_z.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('empty');
  });

  test('test command with -n (non-zero length string)', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nVAL=something\nif [ -n \\\"$VAL\\\" ]; then\n  echo nonempty\nfi" > /tmp/test_n.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_n.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('nonempty');
  });

  test('test command with logical AND (-a)', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nif [ 3 -lt 5 -a 7 -gt 5 ]; then\n  echo both-true\nfi" > /tmp/test_and.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_and.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('both-true');
  });

  test('test command with logical OR (-o)', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nif [ 3 -lt 5 -o 3 -gt 5 ]; then\n  echo one-true\nfi" > /tmp/test_or.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_or.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('one-true');
  });

  test('arithmetic expansion in control flow', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\nRESULT=$((2 + 3))\nif [ $RESULT -eq 5 ]; then\n  echo correct\nfi" > /tmp/arith_if.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/arith_if.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('correct');
  });
});
