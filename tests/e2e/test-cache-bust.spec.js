const { test } = require('@playwright/test');

test('Cache bust test', async ({ page, context }) => {
  // Clear all cache before test
  await context.clearCookies();

  // Navigate with no-cache
  await page.goto('http://127.0.0.1:8000', {
    waitUntil: 'networkidle',
  });

  // Wait for page load
  await page.waitForTimeout(2000);

  // Check if WASM loaded
  const wasmLoaded = await page.evaluate(() => {
    return typeof window.wasmModule !== 'undefined';
  });

  console.log(`WASM loaded: ${wasmLoaded}`);

  // Try a simple echo with single quotes
  const input = page.locator('#terminal-input');
  await input.fill("echo 'test $VAR'");
  await input.press('Enter');
  await page.waitForTimeout(500);

  // Get output
  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  for (let i = 0; i < count; i++) {
    const text = await outputs.nth(i).textContent();
    console.log(`Output ${i}: "${text}"`);
  }
});
