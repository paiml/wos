// Debug test to check ls output without flags
const { test, expect } = require('@playwright/test');

test('debug simple ls output after touch', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000');
  await page.waitForTimeout(2000);

  // Dismiss tutorial if present
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Execute touch command
  await page.locator('#terminal-input').fill('touch testfile.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Try different ls variations
  await page.locator('#terminal-input').fill('ls');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Get terminal output
  const terminalOutput = await page.locator('#terminal-output').textContent();
  console.log('=== TERMINAL OUTPUT (ls) ===');
  console.log(terminalOutput);
  console.log('=== END OUTPUT ===');
});
