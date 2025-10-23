// Vim :help Command Test (WOS-402)
// Bug Report: docs/qa/bug-report-cd-terminal-state-vim.md
const { test, expect } = require('@playwright/test');

async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(500);
}

async function executeVimCommand(page, command) {
  await page.keyboard.press(':');
  await page.waitForTimeout(100);
  await page.keyboard.type(command);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(500);
}

async function openVim(page, filename) {
  await executeCommand(page, `vim ${filename}`);
  await page.waitForTimeout(1000);
}

async function getVimMessage(page) {
  const message = page.locator('.vim-message');
  if (await message.isVisible()) {
    return await message.textContent();
  }
  const vimContainer = page.locator('#vim-editor-container');
  if (await vimContainer.isVisible()) {
    return await vimContainer.textContent();
  }
  return '';
}

test.describe('Vim :help Command (WOS-402)', () => {
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

  test(':help command shows available commands', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'help');

    const message = await getVimMessage(page);
    console.log(`:help output: "${message}"`);

    // Should show help text with available commands
    expect(message).toMatch(/:w|write/i);
    expect(message).toMatch(/:q|quit/i);
    expect(message).toMatch(/:wq/i);

    await page.screenshot({
      path: 'tests/e2e/screenshots/VIM-HELP-01-help-command.png',
      fullPage: true
    });

    console.log('✅ :help shows available commands');
  });

  test(':help lists all available vim commands', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'help');

    const message = await getVimMessage(page);

    // Should list all 8 commands
    expect(message).toMatch(/:w/);
    expect(message).toMatch(/:write/);
    expect(message).toMatch(/:q/);
    expect(message).toMatch(/:quit/);
    expect(message).toMatch(/:q!/);
    expect(message).toMatch(/:wq/);
    expect(message).toMatch(/:x/);

    console.log('✅ :help lists all commands');
  });

  test(':help includes command descriptions', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'help');

    const message = await getVimMessage(page);

    // Should have descriptions or usage info
    expect(message.length).toBeGreaterThan(100); // Substantial help text

    await page.screenshot({
      path: 'tests/e2e/screenshots/VIM-HELP-02-with-descriptions.png',
      fullPage: true
    });

    console.log('✅ :help includes descriptions');
  });

  test(':help can be dismissed with Escape', async ({ page }) => {
    await openVim(page, 'test.txt');

    await executeVimCommand(page, 'help');
    await page.waitForTimeout(500);

    // Press Escape to clear help
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    const message = await getVimMessage(page);
    console.log(`Message after Escape: "${message}"`);

    // Help should be cleared (or at least different)
    expect(message.length).toBeLessThan(50); // Short or empty

    console.log('✅ :help can be dismissed');
  });

  test(':help then :w still works', async ({ page }) => {
    await openVim(page, 'test.txt');

    // Show help
    await executeVimCommand(page, 'help');
    await page.waitForTimeout(500);

    // Then save file
    await page.keyboard.press('Escape');
    await executeVimCommand(page, 'w');

    const message = await getVimMessage(page);
    console.log(`Message after :w: "${message}"`);

    // Should show write confirmation
    expect(message).toMatch(/written/i);

    console.log('✅ Commands still work after :help');
  });
});
