// Help Menu Test - Verify simplified menu with paiml.com link
const { test, expect } = require('@playwright/test');

test.describe('Help Menu', () => {
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

  test('help menu shows "Made by Pragmatic AI Labs" link', async ({ page }) => {
    // Click help button (green circle icon)
    const helpButton = page.locator('#btn-help');
    await helpButton.click();
    await page.waitForTimeout(300);

    // Verify menu is visible
    const helpMenu = page.locator('#help-menu');
    await expect(helpMenu).toBeVisible();

    // Verify header is "About"
    const header = page.locator('#help-menu h3');
    await expect(header).toHaveText('About');

    // Verify single link exists
    const link = page.locator('#help-menu a.help-menu-item');
    await expect(link).toBeVisible();
    await expect(link).toHaveText('Made by Pragmatic AI Labs');

    // Verify link goes to paiml.com
    const href = await link.getAttribute('href');
    expect(href).toBe('https://paiml.com');

    // Verify security attributes
    const rel = await link.getAttribute('rel');
    expect(rel).toContain('noopener');
    expect(rel).toContain('noreferrer');

    const target = await link.getAttribute('target');
    expect(target).toBe('_blank');

    await page.screenshot({
      path: 'tests/e2e/screenshots/HELP-01-simplified-menu.png',
      fullPage: true
    });

    console.log('✅ Help menu simplified to single paiml.com link');
  });

  test('no 404 links in help menu', async ({ page }) => {
    // Click help button
    const helpButton = page.locator('#btn-help');
    await helpButton.click();
    await page.waitForTimeout(300);

    // Count all links in menu
    const links = await page.locator('#help-menu a').count();
    console.log(`Help menu link count: ${links}`);

    // Should have exactly 1 link (paiml.com)
    expect(links).toBe(1);

    // No buttons (like retake tutorial) should exist
    const buttons = await page.locator('#help-menu button').count();
    console.log(`Help menu button count: ${buttons}`);
    expect(buttons).toBe(0);

    console.log('✅ No 404 links - only valid paiml.com link');
  });
});
