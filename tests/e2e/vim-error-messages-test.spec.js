// Vim Error Messages Test (WOS-401)
// Bug Report: docs/qa/bug-report-cd-terminal-state-vim.md
const { test, expect } = require('@playwright/test');

// Helper to execute terminal command
async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(500);
}

// Helper to execute vim command (in command mode)
async function executeVimCommand(page, command) {
  await page.keyboard.press(':');
  await page.waitForTimeout(100);
  await page.keyboard.type(command);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(500);
}

// Helper to open vim
async function openVim(page, filename) {
  await executeCommand(page, `vim ${filename}`);
  await page.waitForTimeout(1000);
}

// Helper to get vim message/status line
async function getVimMessage(page) {
  // Vim message appears in .vim-message or similar
  const message = page.locator('.vim-message');
  if (await message.isVisible()) {
    return await message.textContent();
  }

  // Fallback: check for error in vim container
  const vimContainer = page.locator('#vim-editor-container');
  if (await vimContainer.isVisible()) {
    const text = await vimContainer.textContent();
    return text;
  }

  return '';
}

test.describe('Vim Error Messages (WOS-401)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://127.0.0.1:8000');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForTimeout(2000);

    // Dismiss tutorial if present
    const skipButton = page.locator('button:has-text("Skip Tutorial")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('unknown vim command shows available commands list', async ({ page }) => {
    await openVim(page, 'test.txt');

    // Try unknown command :sp (split window)
    await executeVimCommand(page, 'sp');

    const message = await getVimMessage(page);
    console.log(`Vim error message: "${message}"`);

    // Should show available commands, not just generic error
    expect(message).toMatch(/available/i);
    expect(message).toMatch(/:w|:q|:wq/); // Should list some available commands

    await page.screenshot({
      path: 'tests/e2e/screenshots/VIM-ERROR-01-sp-command.png',
      fullPage: true
    });

    console.log('✅ Unknown command shows helpful error with available commands');
  });

  test('vim error for :vs shows command list', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'vs');

    const message = await getVimMessage(page);
    console.log(`Vim error for :vs: "${message}"`);

    expect(message).toMatch(/available|supported|command/i);

    await page.screenshot({
      path: 'tests/e2e/screenshots/VIM-ERROR-02-vs-command.png',
      fullPage: true
    });

    console.log('✅ :vs error is helpful');
  });

  test('vim error for :split shows command list', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'split');

    const message = await getVimMessage(page);
    console.log(`Vim error for :split: "${message}"`);

    expect(message).toMatch(/available|supported|command/i);

    console.log('✅ :split error is helpful');
  });

  test('vim error for :help shows either help or available commands', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'help');

    const message = await getVimMessage(page);
    console.log(`Vim response to :help: "${message}"`);

    // Either shows help content OR says command not available with list
    expect(message.length).toBeGreaterThan(0);

    await page.screenshot({
      path: 'tests/e2e/screenshots/VIM-ERROR-03-help-command.png',
      fullPage: true
    });

    console.log('✅ :help shows something useful');
  });

  test('vim error for :e shows command list', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'e newfile.txt');

    const message = await getVimMessage(page);
    console.log(`Vim error for :e: "${message}"`);

    expect(message).toMatch(/available|supported|command/i);

    console.log('✅ :e error is helpful');
  });

  test('vim error for :set shows command list', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'set number');

    const message = await getVimMessage(page);
    console.log(`Vim error for :set: "${message}"`);

    expect(message).toMatch(/available|supported|command/i);

    console.log('✅ :set error is helpful');
  });

  test('known vim commands still work', async ({ page }) => {
    await openVim(page, 'test.txt');

    // Type some content
    await page.keyboard.press('i'); // Insert mode
    await page.keyboard.type('Hello World');
    await page.keyboard.press('Escape');

    // Save with :w
    await executeVimCommand(page, 'w');

    const message = await getVimMessage(page);
    console.log(`Vim message after :w: "${message}"`);

    // Should not show error for known command
    expect(message).not.toMatch(/not.*command|unknown|error/i);

    await page.screenshot({
      path: 'tests/e2e/screenshots/VIM-ERROR-04-known-command-w.png',
      fullPage: true
    });

    console.log('✅ Known command :w still works');
  });

  test('quit command still works', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'q!');

    // Should close vim and return to terminal
    await page.waitForTimeout(500);

    const vimContainer = page.locator('#vim-editor-container');
    const isVisible = await vimContainer.isVisible();

    expect(isVisible).toBe(false);

    console.log('✅ Known command :q! still works');
  });

  test('error message format is consistent', async ({ page }) => {
    await openVim(page, 'test.txt');

    const commands = ['sp', 'vs', 'vsplit', 'tabnew', 'buffers'];
    const messages = [];

    for (const cmd of commands) {
      await executeVimCommand(page, cmd);
      const message = await getVimMessage(page);
      messages.push(message);
      console.log(`Error for :${cmd}: "${message}"`);
    }

    // All error messages should follow similar format
    const allHaveAvailable = messages.every(msg =>
      msg.match(/available|supported|command/i)
    );

    expect(allHaveAvailable).toBe(true);

    await page.screenshot({
      path: 'tests/e2e/screenshots/VIM-ERROR-05-consistent-format.png',
      fullPage: true
    });

    console.log('✅ All error messages have consistent format');
  });

  test('error message lists :w :q :wq as available', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'nonexistent');

    const message = await getVimMessage(page);
    console.log(`Available commands in error: "${message}"`);

    // Should specifically mention the core commands
    expect(message).toMatch(/:w/);
    expect(message).toMatch(/:q/);
    expect(message).toMatch(/:wq/);

    await page.screenshot({
      path: 'tests/e2e/screenshots/VIM-ERROR-06-lists-core-commands.png',
      fullPage: true
    });

    console.log('✅ Error lists core commands :w :q :wq');
  });
});
