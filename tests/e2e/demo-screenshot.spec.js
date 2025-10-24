// Quick demo screenshot
const { test } = require('@playwright/test');

test('capture working accordion demo', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000');

  // Clear localStorage for clean state
  await page.evaluate(() => localStorage.clear());
  await page.reload();

  await page.waitForTimeout(3000);

  // Dismiss tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Capture initial state
  await page.screenshot({
    path: 'tests/e2e/screenshots/DEMO-01-initial.png',
    fullPage: true
  });

  // Expand process list
  const processPanel = page.locator('[data-panel="process_list"]');
  await processPanel.locator('.btn-collapse').click({ force: true });
  await page.waitForTimeout(500);

  await page.screenshot({
    path: 'tests/e2e/screenshots/DEMO-02-process-list-expanded.png',
    fullPage: true
  });

  // Expand memory map (should collapse process list)
  const memoryPanel = page.locator('[data-panel="memory_map"]');
  await memoryPanel.locator('.btn-collapse').click({ force: true });
  await page.waitForTimeout(500);

  await page.screenshot({
    path: 'tests/e2e/screenshots/DEMO-03-memory-map-expanded.png',
    fullPage: true
  });

  console.log('✅ Demo screenshots captured!');
});
