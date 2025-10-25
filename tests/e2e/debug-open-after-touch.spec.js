// Debug test - open Files panel AFTER touch command
const { test, expect } = require('@playwright/test');

test('open files panel after touch', async ({ page, context }) => {
  await context.clearCookies();
  await page.goto('http://127.0.0.1:8000', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);

  // Dismiss tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Execute touch command BEFORE opening Files panel
  await page.locator('#terminal-input').fill('touch testfile.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // NOW open Files panel
  await page.locator('button[data-panel-toggle="filesystem"]').click();
  await page.waitForTimeout(500);

  // Check file list
  const fileItems = await page.locator('.file-item').count();
  console.log(`\nFile items: ${fileItems}`);
  for (let i = 0; i < fileItems; i++) {
    const nameSpan = page.locator('.file-item').nth(i).locator('.file-item-name span');
    const name = await nameSpan.textContent();
    console.log(`  File ${i}: "${name}"`);
  }

  // Take screenshot
  await page.screenshot({ path: 'test-results/debug-open-after-touch.png', fullPage: true });
});
