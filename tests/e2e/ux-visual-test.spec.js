// Visual UX Testing with Playwright
// Takes screenshots to evaluate actual UI appearance

const { test, expect } = require('@playwright/test');

test.describe('WOS Panel UX Visual Testing', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the local dev server
    await page.goto('http://127.0.0.1:8000');

    // Wait for WASM to load
    await page.waitForTimeout(2000);

    // Dismiss tutorial modal if present
    const skipButton = page.locator('button:has-text("Skip Tutorial")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('screenshot initial state - all panels', async ({ page }) => {
    // Take full page screenshot
    await page.screenshot({
      path: 'tests/e2e/screenshots/01-initial-state.png',
      fullPage: true
    });

    console.log('Screenshot saved: 01-initial-state.png');
  });

  test('screenshot collapsed panels', async ({ page }) => {
    // Find all collapse buttons and click them to collapse all panels
    const collapseButtons = await page.locator('.btn-collapse').all();

    for (const btn of collapseButtons) {
      await btn.click();
      await page.waitForTimeout(300); // Wait for animation
    }

    await page.screenshot({
      path: 'tests/e2e/screenshots/02-all-collapsed.png',
      fullPage: true
    });

    console.log('Screenshot saved: 02-all-collapsed.png');
  });

  test('screenshot expand first panel', async ({ page }) => {
    // Collapse all first
    const collapseButtons = await page.locator('.btn-collapse').all();
    for (const btn of collapseButtons) {
      await btn.click();
      await page.waitForTimeout(100);
    }

    // Expand first panel
    const firstPanel = page.locator('[data-panel]').first();
    const firstCollapseBtn = firstPanel.locator('.btn-collapse');
    await firstCollapseBtn.click();
    await page.waitForTimeout(300);

    await page.screenshot({
      path: 'tests/e2e/screenshots/03-first-panel-expanded.png',
      fullPage: true
    });

    console.log('Screenshot saved: 03-first-panel-expanded.png');
  });

  test('screenshot expand different panels sequentially', async ({ page }) => {
    const panels = await page.locator('[data-panel]').all();

    let index = 1;
    for (const panel of panels.slice(0, 3)) {  // Test first 3 panels
      const panelName = await panel.getAttribute('data-panel');
      const collapseBtn = panel.locator('.btn-collapse');

      await collapseBtn.click();
      await page.waitForTimeout(300);

      await page.screenshot({
        path: `tests/e2e/screenshots/04-panel-${index}-${panelName}.png`,
        fullPage: true
      });

      console.log(`Screenshot saved: 04-panel-${index}-${panelName}.png`);
      index++;
    }
  });

  test('measure panel heights and spacing', async ({ page }) => {
    const panels = await page.locator('[data-panel]').all();

    const measurements = [];
    for (const panel of panels) {
      const box = await panel.boundingBox();
      const panelName = await panel.getAttribute('data-panel');
      const isCollapsed = await panel.evaluate(el => el.classList.contains('collapsed'));

      measurements.push({
        panel: panelName,
        collapsed: isCollapsed,
        height: box ? box.height : 0,
        y: box ? box.y : 0
      });
    }

    console.log('Panel measurements:', JSON.stringify(measurements, null, 2));

    // Check if panels are off-screen
    const viewport = page.viewportSize();
    const offscreen = measurements.filter(m => m.y + m.height > viewport.height);

    if (offscreen.length > 0) {
      console.log('PROBLEM: Panels extending off-screen:', offscreen);
    }
  });

  test('test accordion behavior visually', async ({ page }) => {
    // Test that expanding one panel collapses others
    const processListPanel = page.locator('[data-panel="process_list"]');
    const memoryMapPanel = page.locator('[data-panel="memory_map"]');

    // Expand process list
    await processListPanel.locator('.btn-collapse').click();
    await page.waitForTimeout(300);

    await page.screenshot({
      path: 'tests/e2e/screenshots/05-accordion-process-list.png',
      fullPage: true
    });

    // Now expand memory map - should collapse process list
    await memoryMapPanel.locator('.btn-collapse').click();
    await page.waitForTimeout(300);

    await page.screenshot({
      path: 'tests/e2e/screenshots/06-accordion-memory-map.png',
      fullPage: true
    });

    // Check if process list is actually collapsed
    const processListCollapsed = await processListPanel.evaluate(el =>
      el.classList.contains('collapsed')
    );

    console.log('Process list collapsed after expanding memory map:', processListCollapsed);
    expect(processListCollapsed).toBe(true);
  });
});
