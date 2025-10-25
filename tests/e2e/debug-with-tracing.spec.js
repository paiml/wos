// Debug test with WOS tracing enabled
const { test, expect } = require('@playwright/test');

test('debug with FILE tracing', async ({ page, context }) => {
  await context.clearCookies();

  // Capture console messages
  const consoleMsgs = [];
  page.on('console', msg => {
    consoleMsgs.push(`[${msg.type()}] ${msg.text()}`);
  });

  // Navigate with FILE tracing enabled
  await page.goto('http://127.0.0.1:8000/?trace=DEBUG&categories=FILE', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);

  // Dismiss tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Run touch
  await page.locator('#terminal-input').fill('touch testfile.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(1000);

  // Open Files panel
  await page.locator('button[data-panel-toggle="filesystem"]').click();
  await page.waitForTimeout(1000);

  // Print all console messages with FILE category
  console.log('\n=== CONSOLE OUTPUT ===');
  consoleMsgs.filter(msg => msg.includes('[FILE]')).forEach(msg => console.log(msg));
  console.log('=== END CONSOLE ===\n');

  // Check file list
  const fileItems = await page.locator('.file-item').count();
  console.log(`File items in DOM: ${fileItems}`);

  for (let i = 0; i < fileItems; i++) {
    const nameSpan = page.locator('.file-item').nth(i).locator('.file-item-name span');
    const name = await nameSpan.textContent();
    console.log(`  File ${i}: "${name}"`);
  }
});
