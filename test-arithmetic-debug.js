// Debug script to see exact arithmetic output
const { chromium } = require('@playwright/test');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  await page.goto('http://127.0.0.1:8000');
  await page.waitForTimeout(2000);

  // Skip tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Execute arithmetic command
  const input = page.locator('#terminal-input');
  await input.fill('echo $((2 + 3))');
  await input.press('Enter');
  await page.waitForTimeout(300);

  // Get output
  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  const lastOutput = outputs.nth(count - 1);
  const text = await lastOutput.textContent();

  console.log('Raw output:', JSON.stringify(text));
  console.log('Output length:', text.length);
  console.log('Output bytes:', Buffer.from(text).toString('hex'));
  console.log('After strip:', JSON.stringify(text.replace(/\n$/, '')));

  await browser.close();
})();
