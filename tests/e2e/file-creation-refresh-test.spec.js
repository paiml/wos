// WOS-FILE-EDIT-01: File Creation and Auto-Refresh E2E Tests
// Tests that file list auto-refreshes after touch command and edit button enables

const { test, expect } = require('@playwright/test');

test.describe('File Creation and Auto-Refresh (WOS-FILE-EDIT-01)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://127.0.0.1:8000');

    // Wait for WASM to load
    await page.waitForTimeout(2000);

    // Dismiss tutorial if present
    const skipButton = page.locator('button:has-text("Skip Tutorial")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('file list auto-refreshes after touch command', async ({ page }) => {
    // Click Files icon to show file panel
    await page.locator('button[data-panel-toggle="filesystem"]').click();
    await page.waitForTimeout(200);

    // Execute touch command to create new file
    await page.locator('#terminal-input').fill('touch newfile.txt');
    await page.locator('#terminal-input').press('Enter');

    // Wait for file system to process and refresh (600ms polling interval + buffer)
    await page.waitForTimeout(1500);

    // Verify file appears in file list
    const fileItem = page.locator('.file-item').filter({ hasText: 'newfile.txt' });
    await expect(fileItem).toBeVisible({ timeout: 2000 });
  });

  test('edit button enables after touch command', async ({ page }) => {
    // Click Files icon to show file panel
    await page.locator('button[data-panel-toggle="filesystem"]').click();
    await page.waitForTimeout(200);

    // Create file
    await page.locator('#terminal-input').fill('touch test.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(1500);

    // WOS-FILE-EDIT-01: File should appear and be auto-selected, edit button enabled
    const fileItem = page.locator('.file-item').filter({ hasText: 'test.txt' });
    await expect(fileItem).toBeVisible({ timeout: 2000 });
    const editBtn = page.locator('#btn-edit');
    await expect(editBtn).toBeEnabled({ timeout: 2000 });
  });

  test('multiple file creation refreshes list correctly', async ({ page }) => {
    // Click Files icon to show file panel
    await page.locator('button[data-panel-toggle="filesystem"]').click();
    await page.waitForTimeout(200);

    // WOS-FILE-EDIT-01: WOS touch doesn't support multiple args, use separate commands
    // Create multiple files with separate commands
    await page.locator('#terminal-input').fill('touch file1.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(500);

    await page.locator('#terminal-input').fill('touch file2.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(500);

    await page.locator('#terminal-input').fill('touch file3.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(500);

    // Verify all files appear
    await expect(page.locator('.file-item').filter({ hasText: 'file1.txt' })).toBeVisible({ timeout: 2000 });
    await expect(page.locator('.file-item').filter({ hasText: 'file2.txt' })).toBeVisible({ timeout: 2000 });
    await expect(page.locator('.file-item').filter({ hasText: 'file3.txt' })).toBeVisible({ timeout: 2000 });
  });

  test('file deletion updates list', async ({ page }) => {
    // Click Files icon to show file panel
    await page.locator('button[data-panel-toggle="filesystem"]').click();
    await page.waitForTimeout(200);

    // Create and then delete a file
    await page.locator('#terminal-input').fill('touch deleteme.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(1500);

    // Verify file exists
    await expect(page.locator('.file-item').filter({ hasText: 'deleteme.txt' })).toBeVisible({ timeout: 2000 });

    // Delete file
    await page.locator('#terminal-input').fill('rm deleteme.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(1500);

    // Verify file removed from list
    await expect(page.locator('.file-item').filter({ hasText: 'deleteme.txt' })).not.toBeVisible({ timeout: 2000 });
  });

  test('file creation with echo redirect updates list', async ({ page }) => {
    // Click Files icon to show file panel
    await page.locator('button[data-panel-toggle="filesystem"]').click();
    await page.waitForTimeout(200);

    // Create file with content using redirect
    await page.locator('#terminal-input').fill('echo "hello" > created.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(1500);

    // Verify file appears
    await expect(page.locator('.file-item').filter({ hasText: 'created.txt' })).toBeVisible({ timeout: 2000 });

    // Verify edit button enabled
    await expect(page.locator('#btn-edit')).toBeEnabled({ timeout: 2000 });
  });
});
