// Terminal Prompt with Current Working Directory Test (WOS-400)
// Bug Report: docs/qa/bug-report-cd-terminal-state-vim.md
const { test, expect } = require('@playwright/test');

// Helper function to execute a command and wait for output
async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(500); // Allow command to execute
}

// Helper function to get the last command prompt
async function getLastPrompt(page) {
  const commandLines = page.locator('.terminal-line.command');
  const count = await commandLines.count();
  if (count === 0) return null;
  return commandLines.nth(count - 1);
}

test.describe('Terminal Prompt with CWD (WOS-400)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://127.0.0.1:8000');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForTimeout(2000);

    // Dismiss tutorial
    const skipButton = page.locator('button:has-text("Skip Tutorial")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('prompt shows current directory on initial load', async ({ page }) => {
    // Execute a command to trigger prompt
    await executeCommand(page, 'ls');

    // Get the prompt for the ls command
    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`Prompt text: "${promptText}"`);

    // Should show user@wos:/path$ format (root directory initially)
    expect(promptText).toMatch(/\w+@wos:\/\$\s+ls/);
    // OR accept simpler format: wos:/$ ls
    expect(promptText).toMatch(/wos:\/\$/);

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-01-initial-root.png',
      fullPage: true
    });

    console.log('✅ Initial prompt shows root directory');
  });

  test('cd command updates prompt to show new directory', async ({ page }) => {
    // Change to /home directory
    await executeCommand(page, 'cd /home');

    // Execute another command to see the updated prompt
    await executeCommand(page, 'ls');

    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`Prompt after cd /home: "${promptText}"`);

    // Should show /home in prompt
    expect(promptText).toMatch(/wos:\/home\$/);

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-02-after-cd-home.png',
      fullPage: true
    });

    console.log('✅ Prompt updated after cd to /home');
  });

  test('cd with absolute path updates prompt correctly', async ({ page }) => {
    // Change to /bin directory
    await executeCommand(page, 'cd /bin');
    await executeCommand(page, 'pwd');

    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`Prompt after cd /bin: "${promptText}"`);

    expect(promptText).toMatch(/wos:\/bin\$/);

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-03-absolute-path.png',
      fullPage: true
    });

    console.log('✅ Absolute path cd updates prompt');
  });

  test('cd with relative path updates prompt correctly', async ({ page }) => {
    // Start at root, cd to home, then to relative path
    await executeCommand(page, 'cd /home');
    await executeCommand(page, 'cd user'); // Relative path

    // Execute command to see prompt
    await executeCommand(page, 'pwd');

    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`Prompt after cd user: "${promptText}"`);

    // Should show /home/user
    expect(promptText).toMatch(/wos:\/home\/user\$/);

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-04-relative-path.png',
      fullPage: true
    });

    console.log('✅ Relative path cd updates prompt');
  });

  test('cd to parent directory (..) updates prompt', async ({ page }) => {
    // Navigate to /home/user then back to /home
    await executeCommand(page, 'cd /home/user');
    await executeCommand(page, 'cd ..');

    await executeCommand(page, 'ls');

    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`Prompt after cd ..: "${promptText}"`);

    // Should show /home
    expect(promptText).toMatch(/wos:\/home\$/);

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-05-parent-directory.png',
      fullPage: true
    });

    console.log('✅ cd .. updates prompt to parent directory');
  });

  test('cd with no arguments goes to home and updates prompt', async ({ page }) => {
    // Change to some directory first
    await executeCommand(page, 'cd /tmp');

    // cd with no args should go to HOME (/)
    await executeCommand(page, 'cd');
    await executeCommand(page, 'pwd');

    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`Prompt after cd (no args): "${promptText}"`);

    // Should show root (HOME is set to "/" in shell.rs:49)
    expect(promptText).toMatch(/wos:\/\$/);

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-06-cd-home.png',
      fullPage: true
    });

    console.log('✅ cd with no args updates prompt to home');
  });

  test('pwd command output matches prompt current directory', async ({ page }) => {
    // Change to /bin
    await executeCommand(page, 'cd /bin');

    // Execute pwd and capture output
    await executeCommand(page, 'pwd');

    // Get pwd output (should be in .terminal-line.output)
    const outputLines = page.locator('.terminal-line.output');
    const count = await outputLines.count();
    const pwdOutput = await outputLines.nth(count - 1).textContent();

    // Get prompt
    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`pwd output: "${pwdOutput}"`);
    console.log(`Prompt: "${promptText}"`);

    // Extract path from pwd output and prompt
    const pwdPath = pwdOutput?.trim();
    expect(promptText).toContain(pwdPath);

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-07-pwd-matches-prompt.png',
      fullPage: true
    });

    console.log('✅ pwd output matches prompt CWD');
  });

  test('prompt format includes user and host', async ({ page }) => {
    await executeCommand(page, 'ls');

    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`Full prompt format: "${promptText}"`);

    // Verify format: user@wos:/path$ command
    // User might be "root", "user", or other default
    expect(promptText).toMatch(/\w+@wos:/); // user@wos:
    expect(promptText).toMatch(/wos:\/.*\$/); // wos:/path$

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-08-format-complete.png',
      fullPage: true
    });

    console.log('✅ Prompt includes user@host:path$ format');
  });

  test('multiple cd commands maintain correct prompt state', async ({ page }) => {
    const testPaths = [
      { command: 'cd /bin', expected: '/bin' },
      { command: 'cd /home', expected: '/home' },
      { command: 'cd /tmp', expected: '/tmp' },
      { command: 'cd /', expected: '/' }
    ];

    for (const test of testPaths) {
      await executeCommand(page, test.command);
      await executeCommand(page, 'pwd');

      const prompt = await getLastPrompt(page);
      const promptText = await prompt?.textContent();

      console.log(`After "${test.command}": "${promptText}"`);
      expect(promptText).toContain(test.expected);
    }

    await page.screenshot({
      path: 'tests/e2e/screenshots/PROMPT-09-multiple-cd-commands.png',
      fullPage: true
    });

    console.log('✅ Multiple cd commands maintain correct prompt state');
  });

  test('prompt does not show old hardcoded format', async ({ page }) => {
    await executeCommand(page, 'cd /bin');
    await executeCommand(page, 'ls');

    const prompt = await getLastPrompt(page);
    const promptText = await prompt?.textContent();

    console.log(`Checking for old format in: "${promptText}"`);

    // OLD FORMAT: "wos$ ls" (hardcoded, no path)
    // Should NOT match this - should have path in it
    expect(promptText).not.toMatch(/^wos\$\s/); // Should NOT be just "wos$"
    expect(promptText).toMatch(/wos:.*\$/); // SHOULD have "wos:/path$" format

    console.log('✅ Old hardcoded format not present');
  });
});
