// Debug test to check timing of file creation
const { test, expect } = require('@playwright/test');

test('debug timing after touch', async ({ page, context }) => {
  // Clear all cache and storage
  await context.clearCookies();
  await context.clearPermissions();

  await page.goto('http://127.0.0.1:8000', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000); // Wait for WASM

  // Dismiss tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Execute touch command
  await page.locator('#terminal-input').fill('touch testfile.txt');
  await page.locator('#terminal-input').press('Enter');

  // Wait multiple intervals to see when file appears
  for (let i = 1; i <= 10; i++) {
    await page.waitForTimeout(200);
    const fileItems = await page.locator('.file-item').count();
    const fileList = await page.locator('#file-list').innerHTML();
    console.log(`\n=== After ${i * 200}ms ===`);
    console.log(`File items: ${fileItems}`);
    if (fileItems > 0) {
      for (let j = 0; j < fileItems; j++) {
        const nameSpan = page.locator('.file-item').nth(j).locator('.file-item-name span');
        const name = await nameSpan.textContent();
        console.log(`  File ${j}: "${name}"`);
      }
    }
  }
});
