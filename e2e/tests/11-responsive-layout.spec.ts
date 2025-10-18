import { test, expect } from '@playwright/test';

test.describe('Responsive Layout', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    // Clear localStorage to ensure clean state
    await page.evaluate(() => localStorage.clear());
    await page.reload();

    // Wait for WASM to initialize
    await page.waitForFunction(
      () => {
        const status = document.getElementById('status');
        return status && status.textContent === 'Ready';
      },
      { timeout: 10000 }
    );
  });

  test('should resize interface to fill viewport at 1920x1080', async ({ page }) => {
    // Set viewport to 1080p
    await page.setViewportSize({ width: 1920, height: 1080 });

    // Wait for layout to settle
    await page.waitForTimeout(500);

    // Container should fill the viewport
    const container = page.locator('.container');
    const containerBox = await container.boundingBox();

    expect(containerBox).not.toBeNull();
    expect(containerBox!.width).toBeGreaterThan(1900); // Close to 1920
    expect(containerBox!.height).toBeGreaterThan(1000); // Close to 1080

    // Terminal should resize dynamically
    const terminal = page.locator('#terminal');
    const terminalBox = await terminal.boundingBox();

    expect(terminalBox).not.toBeNull();
    expect(terminalBox!.height).toBeGreaterThan(700); // Should be much larger than fixed 600px
  });

  test('should resize interface when viewport changes from 1080p to 1440p', async ({ page }) => {
    // Start at 1080p
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(300);

    const terminal1080 = page.locator('#terminal');
    const box1080 = await terminal1080.boundingBox();

    // Resize to 1440p
    await page.setViewportSize({ width: 2560, height: 1440 });
    await page.waitForTimeout(300);

    const terminal1440 = page.locator('#terminal');
    const box1440 = await terminal1440.boundingBox();

    // Terminal should be taller at 1440p
    expect(box1440!.height).toBeGreaterThan(box1080!.height);
    expect(box1440!.width).toBeGreaterThan(box1080!.width);
  });

  test('should resize interface when viewport shrinks', async ({ page }) => {
    // Start at 1920x1080
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(300);

    const terminalLarge = page.locator('#terminal');
    const boxLarge = await terminalLarge.boundingBox();

    // Resize to 1280x720
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.waitForTimeout(300);

    const terminalSmall = page.locator('#terminal');
    const boxSmall = await terminalSmall.boundingBox();

    // Terminal should be smaller
    expect(boxSmall!.height).toBeLessThan(boxLarge!.height);
    expect(boxSmall!.width).toBeLessThan(boxLarge!.width);
  });

  test('should maintain responsive behavior when resizing window multiple times', async ({ page }) => {
    const viewports = [
      { width: 1920, height: 1080 },
      { width: 1366, height: 768 },
      { width: 1920, height: 1080 },
      { width: 2560, height: 1440 },
    ];

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      await page.waitForTimeout(300);

      const terminal = page.locator('#terminal');
      const box = await terminal.boundingBox();

      // Terminal should always be visible and have reasonable dimensions
      expect(box).not.toBeNull();
      expect(box!.height).toBeGreaterThan(300); // Minimum usable height
      expect(box!.width).toBeGreaterThan(500); // Minimum usable width
    }
  });

  test('should have container fill 100% of viewport height', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(300);

    const viewportHeight = await page.evaluate(() => window.innerHeight);
    const container = page.locator('.container');
    const containerBox = await container.boundingBox();

    // Container height should match viewport height within 5px tolerance
    expect(Math.abs(containerBox!.height - viewportHeight)).toBeLessThan(5);
  });

  test('should not have horizontal scroll at 1920x1080', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(300);

    const hasHorizontalScroll = await page.evaluate(() => {
      return document.documentElement.scrollWidth > document.documentElement.clientWidth;
    });

    expect(hasHorizontalScroll).toBeFalsy();
  });

  test('should resize file-manager panels to match terminal height', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(300);

    const terminal = page.locator('#terminal');
    const fileManager = page.locator('.file-manager');

    const terminalBox = await terminal.boundingBox();
    const fileManagerBox = await fileManager.boundingBox();

    // File manager and terminal should have similar heights (within 20% tolerance)
    const heightRatio = fileManagerBox!.height / terminalBox!.height;
    expect(heightRatio).toBeGreaterThan(0.8);
    expect(heightRatio).toBeLessThan(1.2);
  });
});
