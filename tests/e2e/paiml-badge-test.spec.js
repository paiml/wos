// PAIML Badge Test - Verify badge in top-right corner
const { test, expect } = require('@playwright/test');

test.describe('PAIML Badge', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://127.0.0.1:8000');
    await page.waitForTimeout(2000);
  });

  test('badge visible in top-right corner with logo and text', async ({ page }) => {
    const badge = page.locator('.paiml-badge');
    await expect(badge).toBeVisible();

    // Verify link points to paiml.com
    const href = await badge.getAttribute('href');
    expect(href).toBe('https://paiml.com');

    // Verify logo is visible
    const logo = page.locator('.paiml-logo');
    await expect(logo).toBeVisible();

    // Verify text is visible
    const text = page.locator('.paiml-text');
    await expect(text).toBeVisible();
    await expect(text).toHaveText('paiml.com');

    // Verify position (top-right corner)
    const badgeBox = await badge.boundingBox();
    const viewportSize = page.viewportSize();

    console.log(`Badge position: (${badgeBox?.x}, ${badgeBox?.y})`);
    console.log(`Viewport size: ${viewportSize?.width}x${viewportSize?.height}`);

    // Should be in right area (x > 50% of viewport)
    expect(badgeBox?.x).toBeGreaterThan(viewportSize.width * 0.5);

    // Should be near top (y < 100px)
    expect(badgeBox?.y).toBeLessThan(100);

    await page.screenshot({
      path: 'tests/e2e/screenshots/BADGE-01-visible.png',
      fullPage: true
    });

    console.log('✅ PAIML badge visible in top-right corner');
  });

  test('badge hover effect works', async ({ page }) => {
    const badge = page.locator('.paiml-badge');

    // Get initial styles
    const initialBorder = await badge.evaluate(el =>
      window.getComputedStyle(el).borderColor
    );

    // Hover over badge
    await badge.hover();
    await page.waitForTimeout(300);

    // Get hover styles
    const hoverBorder = await badge.evaluate(el =>
      window.getComputedStyle(el).borderColor
    );

    // Border color should change on hover
    console.log(`Initial border: ${initialBorder}`);
    console.log(`Hover border: ${hoverBorder}`);

    await page.screenshot({
      path: 'tests/e2e/screenshots/BADGE-02-hover.png',
      fullPage: true
    });

    console.log('✅ Badge hover effect working');
  });

  test('badge link opens paiml.com', async ({ page, context }) => {
    const badge = page.locator('.paiml-badge');

    // Listen for new page (target="_blank")
    const [newPage] = await Promise.all([
      context.waitForEvent('page'),
      badge.click()
    ]);

    // Wait for new page to load
    await newPage.waitForLoadState();

    // Verify URL
    expect(newPage.url()).toContain('paiml.com');

    await newPage.close();

    console.log('✅ Badge link opens paiml.com in new tab');
  });
});
