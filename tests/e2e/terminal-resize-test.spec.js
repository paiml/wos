// Terminal Resize Handle Test
const { test, expect } = require('@playwright/test');

test.describe('Terminal Resize UX', () => {
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

  test('terminal starts at 250px height', async ({ page }) => {
    const terminal = page.locator('.terminal-container');
    const box = await terminal.boundingBox();

    console.log(`Terminal height: ${box?.height}px`);
    expect(box?.height).toBeGreaterThanOrEqual(240); // Allow 10px tolerance
    expect(box?.height).toBeLessThanOrEqual(260);

    await page.screenshot({
      path: 'tests/e2e/screenshots/RESIZE-01-initial.png',
      fullPage: true
    });
  });

  test('resize handle visible in bottom-right corner', async ({ page }) => {
    const resizeHandle = page.locator('.resize-handle');
    await expect(resizeHandle).toBeVisible();

    const handleBox = await resizeHandle.boundingBox();
    const terminalBox = await page.locator('.terminal-container').boundingBox();

    // Verify handle is in bottom-right corner
    expect(handleBox?.x).toBeGreaterThan(terminalBox.x);
    expect(handleBox?.y).toBeGreaterThan(terminalBox.y);

    console.log(`✅ Resize handle positioned at (${handleBox?.x}, ${handleBox?.y})`);

    await page.screenshot({
      path: 'tests/e2e/screenshots/RESIZE-02-handle-visible.png',
      fullPage: true
    });
  });

  test('drag resize handle changes terminal height', async ({ page }) => {
    const resizeHandle = page.locator('.resize-handle');
    const terminal = page.locator('.terminal-container');

    // Get initial height
    const initialBox = await terminal.boundingBox();
    const initialHeight = initialBox?.height;

    console.log(`Initial terminal height: ${initialHeight}px`);

    // Get handle position
    const handleBox = await resizeHandle.boundingBox();
    const startX = handleBox.x + handleBox.width / 2;
    const startY = handleBox.y + handleBox.height / 2;

    // Drag down 100px
    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX, startY + 100, { steps: 10 });
    await page.mouse.up();

    await page.waitForTimeout(300);

    // Get new height
    const newBox = await terminal.boundingBox();
    const newHeight = newBox?.height;

    console.log(`New terminal height: ${newHeight}px (delta: ${newHeight - initialHeight}px)`);

    // Verify height increased
    expect(newHeight).toBeGreaterThan(initialHeight);
    expect(newHeight - initialHeight).toBeGreaterThanOrEqual(90); // Allow some tolerance

    await page.screenshot({
      path: 'tests/e2e/screenshots/RESIZE-03-after-drag.png',
      fullPage: true
    });

    console.log('✅ Terminal resize via drag working');
  });

  test('resize persists in localStorage', async ({ page }) => {
    const resizeHandle = page.locator('.resize-handle');
    const terminal = page.locator('.terminal-container');

    // Drag to new size
    const handleBox = await resizeHandle.boundingBox();
    const startX = handleBox.x + handleBox.width / 2;
    const startY = handleBox.y + handleBox.height / 2;

    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX, startY + 80, { steps: 10 });
    await page.mouse.up();

    await page.waitForTimeout(300);

    // Get resized height
    const resizedBox = await terminal.boundingBox();
    const resizedHeight = resizedBox?.height;

    // Check localStorage
    const savedHeight = await page.evaluate(() => {
      return localStorage.getItem('wos-terminal-height');
    });

    console.log(`Resized height: ${resizedHeight}px`);
    console.log(`Saved in localStorage: ${savedHeight}px`);

    expect(savedHeight).toBeTruthy();
    expect(Math.abs(parseFloat(savedHeight) - resizedHeight)).toBeLessThan(5);

    // Reload page
    await page.reload();
    await page.waitForTimeout(2000);

    // Verify height restored
    const restoredBox = await terminal.boundingBox();
    const restoredHeight = restoredBox?.height;

    console.log(`Restored height after reload: ${restoredHeight}px`);
    expect(Math.abs(restoredHeight - resizedHeight)).toBeLessThan(5);

    console.log('✅ Terminal resize persistence working');
  });

  test('enforce min/max height constraints', async ({ page }) => {
    const resizeHandle = page.locator('.resize-handle');
    const terminal = page.locator('.terminal-container');

    const handleBox = await resizeHandle.boundingBox();
    const startX = handleBox.x + handleBox.width / 2;
    const startY = handleBox.y + handleBox.height / 2;

    // Try to drag way up (should hit 100px minimum)
    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX, startY - 500, { steps: 10 });
    await page.mouse.up();

    await page.waitForTimeout(300);

    const minBox = await terminal.boundingBox();
    console.log(`After dragging up: ${minBox?.height}px (min should be 100px)`);
    expect(minBox?.height).toBeGreaterThanOrEqual(100);

    // Try to drag way down (should hit 600px maximum)
    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX, startY + 1000, { steps: 10 });
    await page.mouse.up();

    await page.waitForTimeout(300);

    const maxBox = await terminal.boundingBox();
    console.log(`After dragging down: ${maxBox?.height}px (max should be 600px)`);
    expect(maxBox?.height).toBeLessThanOrEqual(610); // Allow small tolerance

    console.log('✅ Min/max height constraints working');
  });
});
