// Verify if touch actually creates files
const { test, expect } = require('@playwright/test');

test('verify touch creates file', async ({ page, context }) => {
  await context.clearCookies();
  await page.goto('http://127.0.0.1:8000', { waitUntil: 'networkidle' });
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
  await page.waitForTimeout(500);

  // Run ls to verify
  await page.locator('#terminal-input').fill('ls');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Get terminal output
  const terminalOutput = await page.locator('#terminal-output').textContent();
  console.log('=== TERMINAL OUTPUT ===');
  console.log(terminalOutput);
  console.log('=== END OUTPUT ===');

  // Check if testfile.txt appears in output
  const hasFile = terminalOutput.includes('testfile.txt');
  console.log(`\nDoes terminal output contain 'testfile.txt': ${hasFile}`);
});
