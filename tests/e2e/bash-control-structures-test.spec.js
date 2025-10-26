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
    allOutput += text + '\\n';
  }
  return allOutput.trim();
}

test.describe('Bash Control Structures (WOS-BASH-06)', () => {
  test.beforeEach(async ({ page, context }) => {
    // Clear all caches to ensure fresh WASM load
    await context.clearCookies();

    // Add cache-busting timestamp and disable cache
    const timestamp = Date.now();
    await page.goto(`http://127.0.0.1:8000?t=${timestamp}`, {
      waitUntil: 'networkidle',
    });

    // Clear localStorage and service workers
    await page.evaluate(() => {
      localStorage.clear();
      if ('serviceWorker' in navigator) {
        navigator.serviceWorker.getRegistrations().then(registrations => {
          for (let registration of registrations) {
            registration.unregister();
          }
        });
      }
    });

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
    await executeCommand(page, 'echo "#!/bin/bash\\nif true; then\\n  echo success\\nfi" > /tmp/if_test.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/if_test.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    console.log(`if-then-fi output: "${output}"`);
    expect(output).toContain('success');
  });

  test('if-then-else structure', async ({ page }) => {
    // Test true condition
    await executeCommand(page, 'echo "#!/bin/bash\\nif true; then\\n  echo true-branch\\nelse\\n  echo false-branch\\nfi" > /tmp/if_else.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/if_else.sh');
    await page.waitForTimeout(500);

    const output1 = await getLastOutput(page);
    expect(output1).toContain('true-branch');
    expect(output1).not.toContain('false-branch');

    // Test false condition
    await executeCommand(page, 'echo "#!/bin/bash\\nif false; then\\n  echo true-branch\\nelse\\n  echo false-branch\\nfi" > /tmp/if_else2.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/if_else2.sh');
    await page.waitForTimeout(500);

    const output2 = await getLastOutput(page);
    expect(output2).toContain('false-branch');
    expect(output2).not.toContain('true-branch');
  });

  test('if-elif-else structure', async ({ page }) => {
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/if_elif.sh");
    await executeCommand(page, "echo 'VAL=2' >> /tmp/if_elif.sh");
    await executeCommand(page, "echo 'if [ $VAL -eq 1 ]; then' >> /tmp/if_elif.sh");
    await executeCommand(page, "echo '  echo one' >> /tmp/if_elif.sh");
    await executeCommand(page, "echo 'elif [ $VAL -eq 2 ]; then' >> /tmp/if_elif.sh");
    await executeCommand(page, "echo '  echo two' >> /tmp/if_elif.sh");
    await executeCommand(page, "echo 'else' >> /tmp/if_elif.sh");
    await executeCommand(page, "echo '  echo other' >> /tmp/if_elif.sh");
    await executeCommand(page, "echo 'fi' >> /tmp/if_elif.sh");
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/if_elif.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    console.log(`if-elif-else output: "${output}"`);
    expect(output).toContain('two');
  });

  test('nested if statements', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nif true; then\\n  if true; then\\n    echo nested-success\\n  fi\\nfi" > /tmp/nested_if.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/nested_if.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('nested-success');
  });

  // WHILE loop tests
  test('while loop basic structure', async ({ page }) => {
    // Use single quotes to prevent variable expansion (WOS now respects single quotes!)
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/while_test.sh");
    await executeCommand(page, "echo 'COUNT=0' >> /tmp/while_test.sh");
    await executeCommand(page, "echo 'while [ $COUNT -lt 3 ]; do' >> /tmp/while_test.sh");
    await executeCommand(page, "echo '  echo $COUNT' >> /tmp/while_test.sh");
    await executeCommand(page, "echo '  COUNT=$((COUNT + 1))' >> /tmp/while_test.sh");
    await executeCommand(page, "echo 'done' >> /tmp/while_test.sh");
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
    // Use single quotes to prevent variable expansion
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/while_break.sh");
    await executeCommand(page, "echo 'COUNT=0' >> /tmp/while_break.sh");
    await executeCommand(page, "echo 'while true; do' >> /tmp/while_break.sh");
    await executeCommand(page, "echo '  echo $COUNT' >> /tmp/while_break.sh");
    await executeCommand(page, "echo '  COUNT=$((COUNT + 1))' >> /tmp/while_break.sh");
    await executeCommand(page, "echo '  if [ $COUNT -ge 2 ]; then' >> /tmp/while_break.sh");
    await executeCommand(page, "echo '    break' >> /tmp/while_break.sh");
    await executeCommand(page, "echo '  fi' >> /tmp/while_break.sh");
    await executeCommand(page, "echo 'done' >> /tmp/while_break.sh");
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/while_break.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    expect(output).toContain('0');
    expect(output).toContain('1');
    expect(output).not.toContain('2');
  });

  test('while loop with continue', async ({ page }) => {
    // Use single quotes to prevent variable expansion
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/while_continue.sh");
    await executeCommand(page, "echo 'COUNT=0' >> /tmp/while_continue.sh");
    await executeCommand(page, "echo 'while [ $COUNT -lt 3 ]; do' >> /tmp/while_continue.sh");
    await executeCommand(page, "echo '  COUNT=$((COUNT + 1))' >> /tmp/while_continue.sh");
    await executeCommand(page, "echo '  if [ $COUNT -eq 2 ]; then' >> /tmp/while_continue.sh");
    await executeCommand(page, "echo '    continue' >> /tmp/while_continue.sh");
    await executeCommand(page, "echo '  fi' >> /tmp/while_continue.sh");
    await executeCommand(page, "echo '  echo $COUNT' >> /tmp/while_continue.sh");
    await executeCommand(page, "echo 'done' >> /tmp/while_continue.sh");
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
    // Use single quotes to prevent variable expansion
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/for_list.sh");
    await executeCommand(page, "echo 'for item in one two three; do' >> /tmp/for_list.sh");
    await executeCommand(page, "echo '  echo $item' >> /tmp/for_list.sh");
    await executeCommand(page, "echo 'done' >> /tmp/for_list.sh");
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
    // Use single quotes to prevent variable expansion
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/for_range.sh");
    await executeCommand(page, "echo 'for i in 1 2 3; do' >> /tmp/for_range.sh");
    await executeCommand(page, "echo '  echo $i' >> /tmp/for_range.sh");
    await executeCommand(page, "echo 'done' >> /tmp/for_range.sh");
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/for_range.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    expect(output).toContain('1');
    expect(output).toContain('2');
    expect(output).toContain('3');
  });

  test('for loop with variable expansion', async ({ page }) => {
    // Use single quotes to prevent variable expansion
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/for_var.sh");
    await executeCommand(page, "echo 'FILES=\"file1 file2 file3\"' >> /tmp/for_var.sh");
    await executeCommand(page, "echo 'for f in $FILES; do' >> /tmp/for_var.sh");
    await executeCommand(page, "echo '  echo $f' >> /tmp/for_var.sh");
    await executeCommand(page, "echo 'done' >> /tmp/for_var.sh");
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/for_var.sh');
    await page.waitForTimeout(500);

    const output = await getAllOutput(page);
    expect(output).toContain('file1');
    expect(output).toContain('file2');
    expect(output).toContain('file3');
  });

  test('for loop with break', async ({ page }) => {
    // Use single quotes to prevent variable expansion
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/for_break.sh");
    await executeCommand(page, "echo 'for i in 1 2 3 4 5; do' >> /tmp/for_break.sh");
    await executeCommand(page, "echo '  echo $i' >> /tmp/for_break.sh");
    await executeCommand(page, "echo '  if [ $i -eq 3 ]; then' >> /tmp/for_break.sh");
    await executeCommand(page, "echo '    break' >> /tmp/for_break.sh");
    await executeCommand(page, "echo '  fi' >> /tmp/for_break.sh");
    await executeCommand(page, "echo 'done' >> /tmp/for_break.sh");
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
    // Use single quotes to prevent variable expansion (WOS now respects single quotes!)
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/case_basic.sh");
    await executeCommand(page, "echo 'VAL=two' >> /tmp/case_basic.sh");
    await executeCommand(page, "echo 'case $VAL in' >> /tmp/case_basic.sh");
    await executeCommand(page, "echo '  one) echo 1 ;;' >> /tmp/case_basic.sh");
    await executeCommand(page, "echo '  two) echo 2 ;;' >> /tmp/case_basic.sh");
    await executeCommand(page, "echo '  three) echo 3 ;;' >> /tmp/case_basic.sh");
    await executeCommand(page, "echo 'esac' >> /tmp/case_basic.sh");
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/case_basic.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    console.log(`case statement output: "${output}"`);
    expect(output).toContain('2');
  });

  test('case statement with default pattern', async ({ page }) => {
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/case_default.sh");
    await executeCommand(page, "echo 'VAL=unknown' >> /tmp/case_default.sh");
    await executeCommand(page, "echo 'case $VAL in' >> /tmp/case_default.sh");
    await executeCommand(page, "echo '  one) echo 1 ;;' >> /tmp/case_default.sh");
    await executeCommand(page, "echo '  two) echo 2 ;;' >> /tmp/case_default.sh");
    await executeCommand(page, "echo '  *) echo default ;;' >> /tmp/case_default.sh");
    await executeCommand(page, "echo 'esac' >> /tmp/case_default.sh");
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/case_default.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('default');
  });

  test('case statement with pattern alternatives', async ({ page }) => {
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/case_alt.sh");
    await executeCommand(page, "echo 'VAL=y' >> /tmp/case_alt.sh");
    await executeCommand(page, "echo 'case $VAL in' >> /tmp/case_alt.sh");
    await executeCommand(page, "echo '  y|yes) echo affirmative ;;' >> /tmp/case_alt.sh");
    await executeCommand(page, "echo '  n|no) echo negative ;;' >> /tmp/case_alt.sh");
    await executeCommand(page, "echo 'esac' >> /tmp/case_alt.sh");
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/case_alt.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('affirmative');

    // Test second alternative
    await executeCommand(page, "echo '#!/bin/bash' > /tmp/case_alt2.sh");
    await executeCommand(page, "echo 'VAL=yes' >> /tmp/case_alt2.sh");
    await executeCommand(page, "echo 'case $VAL in' >> /tmp/case_alt2.sh");
    await executeCommand(page, "echo '  y|yes) echo affirmative ;;' >> /tmp/case_alt2.sh");
    await executeCommand(page, "echo '  n|no) echo negative ;;' >> /tmp/case_alt2.sh");
    await executeCommand(page, "echo 'esac' >> /tmp/case_alt2.sh");
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/case_alt2.sh');
    await page.waitForTimeout(500);

    const output2 = await getLastOutput(page);
    expect(output2).toContain('affirmative');
  });

  // Test expression tests
  test('test command with -eq numeric comparison', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nif [ 5 -eq 5 ]; then\\n  echo equal\\nfi" > /tmp/test_eq.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_eq.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('equal');
  });

  test('test command with -lt numeric comparison', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nif [ 3 -lt 5 ]; then\\n  echo less\\nfi" > /tmp/test_lt.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_lt.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('less');
  });

  test('test command with -gt numeric comparison', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nif [ 7 -gt 5 ]; then\\n  echo greater\\nfi" > /tmp/test_gt.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_gt.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('greater');
  });

  test('test command with string comparison', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nif [ \\\"hello\\\" = \\\"hello\\\" ]; then\\n  echo match\\nfi" > /tmp/test_str.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_str.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('match');
  });

  test('test command with -z (zero length string)', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nEMPTY=\\nif [ -z \\\"$EMPTY\\\" ]; then\\n  echo empty\\nfi" > /tmp/test_z.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_z.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('empty');
  });

  test('test command with -n (non-zero length string)', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nVAL=something\\nif [ -n \\\"$VAL\\\" ]; then\\n  echo nonempty\\nfi" > /tmp/test_n.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_n.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('nonempty');
  });

  test('test command with logical AND (-a)', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nif [ 3 -lt 5 -a 7 -gt 5 ]; then\\n  echo both-true\\nfi" > /tmp/test_and.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_and.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('both-true');
  });

  test('test command with logical OR (-o)', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nif [ 3 -lt 5 -o 3 -gt 5 ]; then\\n  echo one-true\\nfi" > /tmp/test_or.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/test_or.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('one-true');
  });

  test('arithmetic expansion in control flow', async ({ page }) => {
    await executeCommand(page, 'echo "#!/bin/bash\\nRESULT=$((2 + 3))\\nif [ $RESULT -eq 5 ]; then\\n  echo correct\\nfi" > /tmp/arith_if.sh');
    await page.waitForTimeout(200);

    await executeCommand(page, 'bash /tmp/arith_if.sh');
    await page.waitForTimeout(500);

    const output = await getLastOutput(page);
    expect(output).toContain('correct');
  });
});
