// Debug test with fresh browser context
const { test, expect } = require('@playwright/test');

test('debug ls with fresh browser', async ({ page, context }) => {
  // Clear storage
  await context.clearCookies();

  await page.goto('http://127.0.0.1:8000');
  await page.waitForTimeout(3000); // Wait longer for WASM

  // Dismiss tutorial if present
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Execute touch command
  await page.locator('#terminal-input').fill('touch testfile.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Check file list DOM
  const fileListHTML = await page.locator('#file-list').innerHTML();
  console.log('=== FILE LIST HTML ===');
  console.log(fileListHTML);
  console.log('=== END HTML ===');

  // Check if file item exists
  const fileItems = await page.locator('.file-item').count();
  console.log(`Found ${fileItems} file items`);

  // Get text content of file items
  for (let i = 0; i < fileItems; i++) {
    const text = await page.locator('.file-item').nth(i).textContent();
    console.log(`File item ${i}: ${text}`);
  }
});
