// Debug panel height issues
const { test } = require('@playwright/test');

test('debug panel dimensions', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000');
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForTimeout(2000);

  // Dismiss tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Click Process List
  await page.locator('[data-panel-toggle="process_list"]').click();
  await page.waitForTimeout(300);

  // Get dimensions
  const fileManager = await page.locator('.file-manager').boundingBox();
  const processPanel = await page.locator('[data-panel="process_list"]').boundingBox();
  const panelHeader = await page.locator('[data-panel="process_list"] .file-panel-header').boundingBox();
  const panelContent = await page.locator('[data-panel="process_list"] .panel-content').boundingBox();

  console.log('\n=== PANEL DIMENSIONS DEBUG ===');
  console.log(`File Manager: ${fileManager?.height}px height`);
  console.log(`Process Panel: ${processPanel?.height}px height`);
  console.log(`Panel Header: ${panelHeader?.height}px height`);
  console.log(`Panel Content: ${panelContent?.height}px height`);
  console.log(`Panel Content visible: ${panelContent !== null}`);

  // Check computed styles
  const panelStyles = await page.locator('[data-panel="process_list"]').evaluate(el => {
    const styles = window.getComputedStyle(el);
    return {
      display: styles.display,
      position: styles.position,
      top: styles.top,
      bottom: styles.bottom,
      height: styles.height,
      overflow: styles.overflow
    };
  });

  console.log('Panel computed styles:', panelStyles);

  await page.screenshot({
    path: 'tests/e2e/screenshots/DEBUG-panel-cutoff.png',
    fullPage: true
  });
});
