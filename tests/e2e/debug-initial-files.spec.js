// Debug test - check what files exist on initial page load
const { test, expect } = require('@playwright/test');

test('check initial filesystem state', async ({ page, context }) => {
  await context.clearCookies();
  await page.goto('http://127.0.0.1:8000', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);

  // Dismiss tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Open Files panel
  await page.locator('button[data-panel-toggle="filesystem"]').click();
  await page.waitForTimeout(500);

  // Check file list
  const fileItems = await page.locator('.file-item').count();
  console.log(`\nInitial file items: ${fileItems}`);

  for (let i = 0; i < fileItems; i++) {
    const nameSpan = page.locator('.file-item').nth(i).locator('.file-item-name span');
    const name = await nameSpan.textContent();
    const isSelected = await page.locator('.file-item').nth(i).evaluate(el => el.classList.contains('selected'));
    console.log(`  File ${i}: "${name}" (selected: ${isSelected})`);
  }

  // Check button state
  const editBtn = page.locator('#btn-edit');
  const isDisabled = await editBtn.isDisabled();
  console.log(`\nEdit button disabled: ${isDisabled}\n`);

  // Take screenshot
  await page.screenshot({ path: 'test-results/debug-initial-files.png', fullPage: true });
});
