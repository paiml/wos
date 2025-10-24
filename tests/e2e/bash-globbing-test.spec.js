// Bash Glob Patterns Test (WOS-BASH-08)
// Reference: GNU Bash Manual - Filename Expansion
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

test.describe('Bash Glob Patterns (WOS-BASH-08)', () => {
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

    // Create test files for globbing
    await executeCommand(page, 'touch /tmp/file1.txt');
    await executeCommand(page, 'touch /tmp/file2.txt');
    await executeCommand(page, 'touch /tmp/file3.txt');
    await executeCommand(page, 'touch /tmp/data.log');
    await executeCommand(page, 'touch /tmp/readme.md');
    await executeCommand(page, 'touch /tmp/test.js');
    await executeCommand(page, 'touch /tmp/app.ts');
    await page.waitForTimeout(300);
  });

  // Asterisk wildcard tests
  test('* matches all files in directory', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/*');
    const output = await getAllOutput(page);
    console.log(`ls /tmp/* output: "${output}"`);
    expect(output).toContain('file1.txt');
    expect(output).toContain('file2.txt');
    expect(output).toContain('file3.txt');
    expect(output).toContain('data.log');
  });

  test('*.txt matches only .txt files', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/*.txt');
    const output = await getAllOutput(page);
    expect(output).toContain('file1.txt');
    expect(output).toContain('file2.txt');
    expect(output).toContain('file3.txt');
    expect(output).not.toContain('data.log');
    expect(output).not.toContain('readme.md');
  });

  test('file*.txt matches file prefix', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/file*.txt');
    const output = await getAllOutput(page);
    expect(output).toContain('file1.txt');
    expect(output).toContain('file2.txt');
    expect(output).toContain('file3.txt');
    expect(output).not.toContain('data.log');
  });

  test('* glob with cat command', async ({ page }) => {
    // Write content to files
    await executeCommand(page, 'echo "content1" > /tmp/test1.txt');
    await executeCommand(page, 'echo "content2" > /tmp/test2.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'cat /tmp/test*.txt');
    const output = await getAllOutput(page);
    expect(output).toContain('content1');
    expect(output).toContain('content2');
  });

  test('glob expands to multiple arguments', async ({ page }) => {
    // Using echo to see expansion
    await executeCommand(page, 'echo /tmp/file*.txt');
    const output = await getLastOutput(page);
    expect(output).toMatch(/file1\.txt.*file2\.txt.*file3\.txt/);
  });

  // Question mark wildcard tests
  test('? matches single character', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/file?.txt');
    const output = await getAllOutput(page);
    expect(output).toContain('file1.txt');
    expect(output).toContain('file2.txt');
    expect(output).toContain('file3.txt');
  });

  test('?? matches exactly two characters', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/ab.txt');
    await executeCommand(page, 'touch /tmp/abc.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/??.txt');
    const output = await getAllOutput(page);
    expect(output).toContain('ab.txt');
    expect(output).not.toContain('abc.txt');
  });

  test('? does not match zero characters', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/.txt');
    await executeCommand(page, 'touch /tmp/a.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/?.txt');
    const output = await getAllOutput(page);
    expect(output).toContain('a.txt');
    expect(output).not.toContain('/tmp/.txt'); // Should not match dot file
  });

  test('combining * and ?', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/file?.t*');
    const output = await getAllOutput(page);
    expect(output).toContain('file1.txt');
    expect(output).toContain('file2.txt');
    expect(output).toContain('file3.txt');
  });

  // Character class tests
  test('[123] matches any of the characters', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/file[123].txt');
    const output = await getAllOutput(page);
    expect(output).toContain('file1.txt');
    expect(output).toContain('file2.txt');
    expect(output).toContain('file3.txt');
  });

  test('[1-3] range matches characters', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/file[1-3].txt');
    const output = await getAllOutput(page);
    expect(output).toContain('file1.txt');
    expect(output).toContain('file2.txt');
    expect(output).toContain('file3.txt');
  });

  test('[a-z] matches lowercase letters', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/fileA.txt');
    await executeCommand(page, 'touch /tmp/filea.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/file[a-z].txt');
    const output = await getAllOutput(page);
    expect(output).toContain('filea.txt');
    expect(output).not.toContain('fileA.txt');
  });

  test('[0-9] matches digits', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/file[0-9].txt');
    const output = await getAllOutput(page);
    expect(output).toContain('file1.txt');
    expect(output).toContain('file2.txt');
    expect(output).toContain('file3.txt');
  });

  test('[!123] negated class - excludes characters', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/file4.txt');
    await executeCommand(page, 'touch /tmp/file5.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/file[!123].txt');
    const output = await getAllOutput(page);
    expect(output).toContain('file4.txt');
    expect(output).toContain('file5.txt');
    expect(output).not.toContain('file1.txt');
    expect(output).not.toContain('file2.txt');
    expect(output).not.toContain('file3.txt');
  });

  test('[^123] negated class (alternative syntax)', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/file4.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/file[^123].txt');
    const output = await getAllOutput(page);
    expect(output).toContain('file4.txt');
    expect(output).not.toContain('file1.txt');
  });

  // Edge cases
  test('glob with no matches returns literal pattern', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/nonexistent*.txt');
    const output = await getLastOutput(page);
    // Should either show error or return literal pattern
    expect(output).toMatch(/no such file|nonexistent/i);
  });

  test('escaped glob characters are literal', async ({ page }) => {
    await executeCommand(page, 'touch "/tmp/file*.txt"');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls "/tmp/file*.txt"');
    const output = await getLastOutput(page);
    expect(output).toContain('file*.txt');
  });

  test('glob in quoted string is not expanded', async ({ page }) => {
    await executeCommand(page, 'echo "*.txt"');
    const output = await getLastOutput(page);
    expect(output).toBe('*.txt'); // Literal, not expanded
  });

  test('multiple globs in one command', async ({ page }) => {
    await executeCommand(page, 'echo /tmp/*.txt /tmp/*.md');
    const output = await getLastOutput(page);
    expect(output).toContain('.txt');
    expect(output).toContain('.md');
  });

  test('glob matches files starting with dot when pattern starts with dot', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/.hidden');
    await executeCommand(page, 'touch /tmp/.config');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/.*');
    const output = await getAllOutput(page);
    expect(output).toContain('.hidden');
    expect(output).toContain('.config');
  });

  test('glob does not match dot files without explicit dot', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/.hidden');
    await executeCommand(page, 'touch /tmp/visible');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/*');
    const output = await getAllOutput(page);
    expect(output).toContain('visible');
    expect(output).not.toContain('.hidden');
  });

  // Complex patterns
  test('complex pattern: *.[jt]s matches .js and .ts files', async ({ page }) => {
    await executeCommand(page, 'ls /tmp/*.[jt]s');
    const output = await getAllOutput(page);
    expect(output).toContain('test.js');
    expect(output).toContain('app.ts');
  });

  test('glob with subdirectories', async ({ page }) => {
    await executeCommand(page, 'mkdir /tmp/subdir');
    await executeCommand(page, 'touch /tmp/subdir/nested.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/*/nested.txt');
    const output = await getAllOutput(page);
    expect(output).toContain('nested.txt');
  });

  // Glob with other commands
  test('glob with rm removes multiple files', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/delete1.tmp');
    await executeCommand(page, 'touch /tmp/delete2.tmp');
    await executeCommand(page, 'touch /tmp/keep.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'rm /tmp/*.tmp');
    await page.waitForTimeout(200);

    await executeCommand(page, 'ls /tmp/');
    const output = await getAllOutput(page);
    expect(output).not.toContain('delete1.tmp');
    expect(output).not.toContain('delete2.tmp');
    expect(output).toContain('keep.txt');
  });

  test('glob with cat concatenates matching files', async ({ page }) => {
    await executeCommand(page, 'echo "line1" > /tmp/part1.txt');
    await executeCommand(page, 'echo "line2" > /tmp/part2.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'cat /tmp/part*.txt');
    const output = await getAllOutput(page);
    expect(output).toContain('line1');
    expect(output).toContain('line2');
  });

  test('glob expansion is sorted alphabetically', async ({ page }) => {
    await executeCommand(page, 'touch /tmp/c.txt');
    await executeCommand(page, 'touch /tmp/a.txt');
    await executeCommand(page, 'touch /tmp/b.txt');
    await page.waitForTimeout(200);

    await executeCommand(page, 'echo /tmp/[abc].txt');
    const output = await getLastOutput(page);
    // Should be sorted: a.txt b.txt c.txt
    const aIndex = output.indexOf('a.txt');
    const bIndex = output.indexOf('b.txt');
    const cIndex = output.indexOf('c.txt');
    expect(aIndex).toBeLessThan(bIndex);
    expect(bIndex).toBeLessThan(cIndex);
  });
});
