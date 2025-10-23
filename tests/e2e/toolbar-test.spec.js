// Icon Toolbar Pattern Test
const { test, expect } = require('@playwright/test');

test.describe('Icon Toolbar UX', () => {
  test.beforeEach(async ({ page }) => {
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
  });

  test('toolbar visible with all icons', async ({ page }) => {
    // Check toolbar exists
    const toolbar = page.locator('.panel-toolbar');
    await expect(toolbar).toBeVisible();

    // Count icons
    const icons = await page.locator('.toolbar-icon').count();
    console.log(`Toolbar icons: ${icons}`);
    expect(icons).toBe(8); // 8 panels

    await page.screenshot({
      path: 'tests/e2e/screenshots/TOOLBAR-01-initial.png',
      fullPage: true
    });
  });

  test('only one panel visible at a time', async ({ page }) => {
    const panels = await page.locator('[data-panel]').all();

    // Count visible panels
    let visibleCount = 0;
    let visiblePanel = null;

    for (const panel of panels) {
      const isVisible = await panel.isVisible();
      if (isVisible) {
        visibleCount++;
        visiblePanel = await panel.getAttribute('data-panel');
      }
    }

    console.log(`Visible panels: ${visibleCount} (${visiblePanel})`);
    expect(visibleCount).toBe(1); // Only 1 panel visible
  });

  test('clicking toolbar switches panels', async ({ page }) => {
    // Click Process List icon
    await page.locator('[data-panel-toggle="process_list"]').click();
    await page.waitForTimeout(300);

    // Verify Process List is visible
    const processPanel = page.locator('[data-panel="process_list"]');
    await expect(processPanel).toBeVisible();

    await page.screenshot({
      path: 'tests/e2e/screenshots/TOOLBAR-02-process-list.png',
      fullPage: true
    });

    // Click Memory Map icon
    await page.locator('[data-panel-toggle="memory_map"]').click();
    await page.waitForTimeout(300);

    // Verify Memory Map is visible, Process List is hidden
    const memoryPanel = page.locator('[data-panel="memory_map"]');
    await expect(memoryPanel).toBeVisible();
    await expect(processPanel).toBeHidden();

    await page.screenshot({
      path: 'tests/e2e/screenshots/TOOLBAR-03-memory-map.png',
      fullPage: true
    });

    console.log('✅ Toolbar panel switching working');
  });

  test('active toolbar button highlighted', async ({ page }) => {
    // Check default active button
    const learningBtn = page.locator('[data-panel-toggle="learning_objectives"]');
    await expect(learningBtn).toHaveClass(/active/);

    // Click different panel
    const processBtn = page.locator('[data-panel-toggle="process_list"]');
    await processBtn.click();
    await page.waitForTimeout(300);

    // Verify active state moved
    await expect(processBtn).toHaveClass(/active/);
    await expect(learningBtn).not.toHaveClass(/active/);

    console.log('✅ Toolbar active state management working');
  });
});
