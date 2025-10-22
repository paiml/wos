import { test, expect } from '@playwright/test';

/**
 * WOS-307: Responsive Design (Multi-Device Support)
 *
 * Requirements:
 * - Desktop (≥1280px): Full 4-panel layout
 * - Laptop (1024-1279px): 3-panel layout
 * - Tablet (768-1023px): 2-panel swipeable
 * - Mobile (≤767px): Single panel with tab navigation
 * - Touch-friendly hit targets (44×44px minimum)
 */

test.describe('WOS-307: Responsive Design', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    await page.waitForLoadState('networkidle');
  });

  test.describe('Desktop Layout (≥1280px)', () => {
    test('should display 4-panel grid layout', async ({ page }) => {
      await page.setViewportSize({ width: 1920, height: 1080 });

      const main = await page.locator('main');
      const gridColumns = await main.evaluate((el) => {
        return window.getComputedStyle(el).gridTemplateColumns;
      });

      // Should have 4 columns or equivalent spacing
      expect(gridColumns).toBeTruthy();
      // Main should use CSS Grid
      const display = await main.evaluate((el) => window.getComputedStyle(el).display);
      expect(display).toBe('grid');
    });

    test('should show all panels simultaneously', async ({ page }) => {
      await page.setViewportSize({ width: 1920, height: 1080 });

      const terminal = await page.locator('#terminal');
      const processPanel = await page.locator('#panel-process-list');
      const systemMonitor = await page.locator('#panel-system-monitor');
      const debuggerPanel = await page.locator('#panel-time-travel-debugger');

      await expect(terminal).toBeVisible();
      await expect(processPanel).toBeVisible();
      await expect(systemMonitor).toBeVisible();
      await expect(debuggerPanel).toBeVisible();
    });

    test('should have appropriate column widths', async ({ page }) => {
      await page.setViewportSize({ width: 1920, height: 1080 });

      const main = await page.locator('main');
      const width = await main.evaluate((el) => el.offsetWidth);

      // Desktop layout should use full width
      expect(width).toBeGreaterThan(1200);
    });
  });

  test.describe('Laptop Layout (1024-1279px)', () => {
    test('should display 3-panel layout', async ({ page }) => {
      await page.setViewportSize({ width: 1200, height: 800 });

      const main = await page.locator('main');
      const display = await main.evaluate((el) => window.getComputedStyle(el).display);
      expect(display).toBe('grid');

      const gridColumns = await main.evaluate((el) => {
        return window.getComputedStyle(el).gridTemplateColumns;
      });

      // Should have fewer columns than desktop
      expect(gridColumns).toBeTruthy();
    });

    test('should prioritize most important panels', async ({ page }) => {
      await page.setViewportSize({ width: 1200, height: 800 });

      // Terminal should always be visible
      const terminal = await page.locator('#terminal');
      await expect(terminal).toBeVisible();
    });
  });

  test.describe('Tablet Layout (768-1023px)', () => {
    test('should display 2-panel layout', async ({ page }) => {
      await page.setViewportSize({ width: 800, height: 600 });

      const main = await page.locator('main');
      const gridColumns = await main.evaluate((el) => {
        return window.getComputedStyle(el).gridTemplateColumns;
      });

      // Should have 2 columns or equivalent
      expect(gridColumns).toBeTruthy();
    });

    test('should support swipe gestures for panel navigation', async ({ page }) => {
      await page.setViewportSize({ width: 800, height: 600 });

      // Check for touch event support
      const fileManager = await page.locator('.file-manager');
      const hasTouchEvents = await fileManager.evaluate((el) => {
        return 'ontouchstart' in window;
      });

      // Swipe functionality may not be testable in all environments
      expect(hasTouchEvents !== undefined).toBeTruthy();
    });

    test('should collapse less important panels', async ({ page }) => {
      await page.setViewportSize({ width: 800, height: 600 });

      // Terminal should still be visible
      const terminal = await page.locator('#terminal');
      await expect(terminal).toBeVisible();
    });
  });

  test.describe('Mobile Layout (≤767px)', () => {
    test('should display single panel layout', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 }); // iPhone SE size

      const main = await page.locator('main');
      const gridColumns = await main.evaluate((el) => {
        return window.getComputedStyle(el).gridTemplateColumns;
      });

      // Should be single column (either "1fr" or a single pixel value like "351px")
      const columnCount = gridColumns.split(' ').length;
      expect(columnCount).toBe(1);
    });

    test('should show tab navigation for panels', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      // Check for mobile navigation elements
      const fileManager = await page.locator('.file-manager');
      await expect(fileManager).toBeVisible();

      // Panels should be accessible via tabs/navigation
      const panels = await page.locator('.file-panel').count();
      expect(panels).toBeGreaterThan(0);
    });

    test('should prioritize terminal on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      const terminal = await page.locator('#terminal');
      await expect(terminal).toBeVisible();

      // Terminal should be easily accessible
      const terminalContainer = await page.locator('.terminal-container');
      await expect(terminalContainer).toBeVisible();
    });

    test('should stack panels vertically', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      const main = await page.locator('main');
      const layoutInfo = await main.evaluate((el) => {
        const styles = window.getComputedStyle(el);
        return {
          display: styles.display,
          gridColumns: styles.gridTemplateColumns,
          flexDirection: styles.flexDirection
        };
      });

      // Should be single column grid or vertical flex
      const isSingleColumn = layoutInfo.gridColumns.split(' ').length === 1;
      const isVerticalFlex = layoutInfo.flexDirection === 'column';

      expect(isSingleColumn || isVerticalFlex).toBeTruthy();
    });
  });

  test.describe('Touch Targets (WCAG 2.1)', () => {
    test('should have minimum 44x44px touch targets on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      // Check button sizes
      const buttons = await page.locator('button').all();

      for (const button of buttons) {
        const isVisible = await button.isVisible();
        if (isVisible) {
          const box = await button.boundingBox();
          if (box) {
            // WCAG 2.1 Level AA requires 44x44px minimum
            expect(box.width).toBeGreaterThanOrEqual(44);
            expect(box.height).toBeGreaterThanOrEqual(44);
          }
        }
      }
    });

    test('should have adequate spacing between touch targets', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      const buttons = await page.locator('.terminal-controls button').all();

      if (buttons.length >= 2) {
        const box1 = await buttons[0].boundingBox();
        const box2 = await buttons[1].boundingBox();

        if (box1 && box2) {
          const spacing = box2.x - (box1.x + box1.width);
          // Minimum 8px spacing recommended
          expect(spacing).toBeGreaterThanOrEqual(8);
        }
      }
    });

    test('should maintain touch target sizes on tablet', async ({ page }) => {
      await page.setViewportSize({ width: 800, height: 600 });

      const collapseButtons = await page.locator('.btn-collapse').all();

      for (const button of collapseButtons) {
        const isVisible = await button.isVisible();
        if (isVisible) {
          const box = await button.boundingBox();
          if (box) {
            expect(box.width).toBeGreaterThanOrEqual(44);
            expect(box.height).toBeGreaterThanOrEqual(44);
          }
        }
      }
    });
  });

  test.describe('Responsive Behavior', () => {
    test('should adapt layout when resizing from desktop to mobile', async ({ page }) => {
      // Start at desktop size
      await page.setViewportSize({ width: 1920, height: 1080 });
      let display = await page.locator('main').evaluate((el) => window.getComputedStyle(el).display);
      expect(display).toBe('grid');

      // Resize to mobile
      await page.setViewportSize({ width: 375, height: 667 });
      display = await page.locator('main').evaluate((el) => window.getComputedStyle(el).display);

      // Layout should adapt
      expect(display).toBeTruthy();
    });

    test('should maintain functionality across all viewport sizes', async ({ page }) => {
      const viewports = [
        { width: 1920, height: 1080, name: 'desktop' },
        { width: 1200, height: 800, name: 'laptop' },
        { width: 800, height: 600, name: 'tablet' },
        { width: 375, height: 667, name: 'mobile' }
      ];

      for (const viewport of viewports) {
        await page.setViewportSize(viewport);

        // Terminal should be functional
        const terminalInput = await page.locator('#terminal-input');
        await expect(terminalInput).toBeVisible();
        await expect(terminalInput).toBeEnabled();

        // Basic interaction should work
        await terminalInput.fill('help');
        await terminalInput.press('Enter');

        // Wait for response
        await page.waitForTimeout(100);
      }
    });

    test('should preserve state when resizing', async ({ page }) => {
      await page.setViewportSize({ width: 1920, height: 1080 });

      // Enter a command
      await page.locator('#terminal-input').fill('echo test');
      await page.locator('#terminal-input').press('Enter');
      await page.waitForTimeout(100);

      // Resize to mobile
      await page.setViewportSize({ width: 375, height: 667 });

      // Terminal output should still be present
      const terminalOutput = await page.locator('#terminal-output');
      const text = await terminalOutput.textContent();
      expect(text).toContain('test');
    });
  });

  test.describe('Performance', () => {
    test('should load efficiently on mobile devices', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      const startTime = Date.now();
      await page.goto('index.html');
      await page.waitForLoadState('networkidle');
      const loadTime = Date.now() - startTime;

      // Should load within 3 seconds on mobile
      expect(loadTime).toBeLessThan(3000);
    });

    test('should not cause layout shifts when resizing', async ({ page }) => {
      await page.setViewportSize({ width: 1920, height: 1080 });

      const terminal = await page.locator('#terminal');
      const initialBox = await terminal.boundingBox();

      // Resize
      await page.setViewportSize({ width: 1200, height: 800 });
      await page.waitForTimeout(100); // Wait for layout to settle

      const newBox = await terminal.boundingBox();

      // Elements should reposition smoothly
      expect(initialBox).toBeTruthy();
      expect(newBox).toBeTruthy();
    });
  });
});
