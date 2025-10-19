import { test, expect } from '@playwright/test';

test.describe('Vim Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    await page.waitForSelector('#terminal-input', { timeout: 10000 });
    await page.waitForFunction(() => {
      const statusText = document.getElementById('status')?.textContent || '';
      return statusText === 'Ready';
    });
  });

  test('should open vim editor modal', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    // Check that vim modal appears
    const vimModal = page.locator('.vim-modal');
    await expect(vimModal).toBeVisible();

    // Check vim header
    const vimFilename = page.locator('.vim-filename');
    await expect(vimFilename).toHaveText('test.txt');

    // Check initial mode
    const vimMode = page.locator('.vim-mode');
    await expect(vimMode).toContainText('NORMAL');
  });

  test('should enter insert mode with "i" key', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });

    // Wait for vim to be ready
    await page.waitForTimeout(100);

    // Press 'i' to enter INSERT mode
    await page.keyboard.press('i');

    // Check mode changed to INSERT
    const vimMode = page.locator('.vim-mode');
    await expect(vimMode).toContainText('INSERT');
  });

  test('should type text in INSERT mode', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode
    await page.keyboard.press('i');

    // Type some text
    await page.keyboard.type('Hello, World!');

    // Check that text appears in editor
    const editorContent = await page.locator('.vim-editor').textContent();
    expect(editorContent).toContain('Hello, World!');
  });

  test('should return to NORMAL mode with Escape', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode
    await page.keyboard.press('i');

    const vimMode = page.locator('.vim-mode');
    await expect(vimMode).toContainText('INSERT');

    // Press Escape to return to NORMAL mode
    await page.keyboard.press('Escape');

    await expect(vimMode).toContainText('NORMAL');
  });

  test('should enter COMMAND mode with ":"', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Press ':' to enter COMMAND mode
    await page.keyboard.press(':');

    // Check mode changed to COMMAND
    const vimMode = page.locator('.vim-mode');
    await expect(vimMode).toContainText('COMMAND');

    // Check command buffer shows ':'
    const vimCommand = page.locator('.vim-command');
    await expect(vimCommand).toHaveText(':');
  });

  test('should save file with :w command', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode and type
    await page.keyboard.press('i');
    await page.keyboard.type('Test content');

    // Return to NORMAL mode
    await page.keyboard.press('Escape');

    // Save file with :w
    await page.keyboard.press(':');
    await page.keyboard.type('w');
    await page.keyboard.press('Enter');

    // Check for success message
    const vimMessage = page.locator('.vim-message');
    await expect(vimMessage).toContainText('written');

    // File should be marked as not modified
    const modifiedIndicator = page.locator('.vim-modified');
    await expect(modifiedIndicator).toHaveClass(/hidden/);
  });

  test('should quit with :q command (no changes)', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Quit with :q (no changes made)
    await page.keyboard.press(':');
    await page.keyboard.type('q');
    await page.keyboard.press('Enter');

    // Vim modal should close
    const vimModal = page.locator('.vim-modal');
    await expect(vimModal).toHaveClass(/hidden/);
  });

  test('should prevent quit with :q if modified', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode and type
    await page.keyboard.press('i');
    await page.keyboard.type('Modified content');
    await page.keyboard.press('Escape');

    // Try to quit with :q
    await page.keyboard.press(':');
    await page.keyboard.type('q');
    await page.keyboard.press('Enter');

    // Should show error message
    const vimMessage = page.locator('.vim-message');
    await expect(vimMessage).toContainText('No write since last change');

    // Vim modal should still be visible
    const vimModal = page.locator('.vim-modal');
    await expect(vimModal).toBeVisible();
  });

  test('should force quit with :q!', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode and type
    await page.keyboard.press('i');
    await page.keyboard.type('Modified content');
    await page.keyboard.press('Escape');

    // Force quit with :q!
    await page.keyboard.press(':');
    await page.keyboard.type('q!');
    await page.keyboard.press('Enter');

    // Vim modal should close
    const vimModal = page.locator('.vim-modal');
    await expect(vimModal).toHaveClass(/hidden/);
  });

  test('should save and quit with :wq', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode and type
    await page.keyboard.press('i');
    await page.keyboard.type('Content to save');
    await page.keyboard.press('Escape');

    // Save and quit with :wq
    await page.keyboard.press(':');
    await page.keyboard.type('wq');
    await page.keyboard.press('Enter');

    // Vim modal should close
    const vimModal = page.locator('.vim-modal');
    await expect(vimModal).toHaveClass(/hidden/);

    // Check terminal shows success message
    await page.waitForTimeout(200);
    const terminalOutput = await page.locator('#terminal-output').textContent();
    // Path is normalized to absolute, so expect /test.txt
    expect(terminalOutput).toContain('File saved: /test.txt');
  });

  test('should navigate with arrow keys', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode and type multiple lines
    await page.keyboard.press('i');
    await page.keyboard.type('Line 1');
    await page.keyboard.press('Enter');
    await page.keyboard.type('Line 2');
    await page.keyboard.press('Enter');
    await page.keyboard.type('Line 3');
    await page.keyboard.press('Escape');

    // Test arrow key navigation
    await page.keyboard.press('ArrowUp');
    await page.keyboard.press('ArrowUp');

    // Position should be at line 1 (row 0)
    const position = page.locator('.vim-position');
    await expect(position).toContainText('1,');
  });

  test('should handle backspace in INSERT mode', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode and type
    await page.keyboard.press('i');
    await page.keyboard.type('Hello, World!');

    // Delete some characters
    await page.keyboard.press('Backspace');
    await page.keyboard.press('Backspace');

    // Check content
    const editorContent = await page.locator('.vim-editor').textContent();
    expect(editorContent).toContain('Hello, Worl');
    expect(editorContent).not.toContain('Hello, World!');
  });

  test('should show modified indicator when file is changed', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Initially not modified
    const modifiedIndicator = page.locator('.vim-modified');
    await expect(modifiedIndicator).toHaveClass(/hidden/);

    // Make a change
    await page.keyboard.press('i');
    await page.keyboard.type('x');
    await page.keyboard.press('Escape');

    // Now should show modified
    await expect(modifiedIndicator).not.toHaveClass(/hidden/);
  });

  test('should close vim with X button', async ({ page }) => {
    const input = page.locator('#terminal-input');

    // Open vim
    await input.fill('vim test.txt');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Click close button
    const closeButton = page.locator('.vim-close');
    await closeButton.click();

    // Vim modal should close
    const vimModal = page.locator('.vim-modal');
    await expect(vimModal).toHaveClass(/hidden/);
  });
});
