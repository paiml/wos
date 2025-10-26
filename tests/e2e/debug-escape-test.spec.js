const { test, expect } = require('@playwright/test');

async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(300);
}

async function getAllOutput(page) {
  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  let allOutput = '';
  for (let i = 0; i < count; i++) {
    const text = await outputs.nth(i).textContent();
    allOutput += text + '\n';
  }
  return allOutput.trim();
}

test('Debug escape sequences', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000');
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForTimeout(2000);

  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Test 1: Single backslash-dollar
  await executeCommand(page, 'echo "\\$TEST"');
  await page.waitForTimeout(200);

  // Test 2: Double backslash-dollar
  await executeCommand(page, 'echo "\\\\$TEST"');
  await page.waitForTimeout(200);

  // Test 3: Write to file with single backslash-dollar
  await executeCommand(page, 'echo "\\$TEST" > /tmp/test1.txt');
  await page.waitForTimeout(200);

  await executeCommand(page, 'cat /tmp/test1.txt');
  await page.waitForTimeout(200);

  // Test 4: Write to file with double backslash-dollar
  await executeCommand(page, 'echo "\\\\$TEST" > /tmp/test2.txt');
  await page.waitForTimeout(200);

  await executeCommand(page, 'cat /tmp/test2.txt');
  await page.waitForTimeout(200);

  // Get all output
  const lines = page.locator('.terminal-line');
  const count = await lines.count();
  console.log(`Total lines: ${count}`);

  for (let i = 0; i < count; i++) {
    const line = lines.nth(i);
    const text = await line.textContent();
    console.log(`Line ${i}: "${text}"`);
  }
});
