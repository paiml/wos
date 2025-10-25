// Debug test to check if page loads and what elements exist
const { test, expect } = require('@playwright/test');

test('check page loads and toolbar exists', async ({ page, context }) => {
  await context.clearCookies();
  await page.goto('http://127.0.0.1:8000', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);

  // Dismiss tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Check if toolbar exists
  const toolbar = await page.locator('.icon-toolbar').count();
  console.log(`Toolbar elements: ${toolbar}`);

  // Check for filesystem button
  const filesystemBtn = await page.locator('button[data-panel-toggle="filesystem"]').count();
  console.log(`Filesystem button count: ${filesystemBtn}`);

  // List all buttons with data-panel-toggle
  const allPanelButtons = await page.locator('button[data-panel-toggle]').count();
  console.log(`Total panel toggle buttons: ${allPanelButtons}`);

  for (let i = 0; i < allPanelButtons; i++) {
    const btn = page.locator('button[data-panel-toggle]').nth(i);
    const toggle = await btn.getAttribute('data-panel-toggle');
    const title = await btn.getAttribute('title');
    console.log(`  Button ${i}: data-panel-toggle="${toggle}", title="${title}"`);
  }

  // Take screenshot
  await page.screenshot({ path: 'test-results/debug-page-load.png', fullPage: true });
});
