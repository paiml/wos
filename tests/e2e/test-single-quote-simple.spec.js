const { test, expect } = require('@playwright/test');

test('test single quote prevents expansion', async ({ page, context }) => {
  // Clear all caches
  await context.clearCookies();

  await page.goto('http://127.0.0.1:8000?t=' + Date.now(), {
    waitUntil: 'networkidle',
  });

  // Clear localStorage
  await page.evaluate(() => localStorage.clear());

  await page.waitForTimeout(2000);

  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Test that single quotes prevent variable expansion
  const input = page.locator('#terminal-input');
  await input.fill("echo 'test $VAR'");
  await input.press('Enter');
  await page.waitForTimeout(500);

  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  const lastOutput = outputs.nth(count - 1);
  const text = await lastOutput.textContent();

  console.log(`Output: "${text}"`);

  // Should output literal "$VAR" not expanded value
  expect(text).toContain('$VAR');
});
