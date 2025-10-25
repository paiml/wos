// Debug test to check ls output after touch command
const { test, expect } = require('@playwright/test');

test('debug ls output after touch', async ({ page }) => {
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

  // Execute ls command and capture output
  await page.locator('#terminal-input').fill('ls -la /');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Take screenshot
  await page.screenshot({ path: 'test-results/debug-ls-output.png', fullPage: true });

  // Get terminal output
  const terminalOutput = await page.locator('#terminal-output').textContent();
  console.log('=== TERMINAL OUTPUT ===');
  console.log(terminalOutput);
  console.log('=== END OUTPUT ===');

  // Check file list DOM
  const fileListHTML = await page.locator('#file-list').innerHTML();
  console.log('=== FILE LIST HTML ===');
  console.log(fileListHTML);
  console.log('=== END HTML ===');
});
