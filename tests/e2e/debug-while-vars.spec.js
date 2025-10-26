// Debug test for while loop variable scoping
const { test, expect } = require('@playwright/test');

async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(300);
}

async function getLastOutput(page) {
  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  if (count === 0) return '';
  const lastOutput = outputs.nth(count - 1);
  return await lastOutput.textContent();
}

test('Debug while loop variable scoping', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000');
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForTimeout(2000);

  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Create the exact test script using the multi-line approach with \\\\$
  await executeCommand(page, 'echo "#!/bin/bash" > /tmp/while_test.sh');
  await executeCommand(page, 'echo "COUNT=0" >> /tmp/while_test.sh');
  await executeCommand(page, 'echo "while [ \\\\$COUNT -lt 3 ]; do" >> /tmp/while_test.sh');
  await executeCommand(page, 'echo "  echo \\\\$COUNT" >> /tmp/while_test.sh');
  await executeCommand(page, 'echo "  COUNT=\\\\$((COUNT + 1))" >> /tmp/while_test.sh');
  await executeCommand(page, 'echo "done" >> /tmp/while_test.sh');
  await page.waitForTimeout(200);

  // Check what was written
  await executeCommand(page, 'cat /tmp/while_test.sh');
  await page.waitForTimeout(500);

  const fileContent = await getLastOutput(page);
  console.log('File content:', fileContent);

  // Now run it
  await executeCommand(page, 'bash /tmp/while_test.sh');
  await page.waitForTimeout(500);

  const output = await getLastOutput(page);
  console.log('bash output:', output);

  // Check browser console for debug logs
  const logs = [];
  page.on('console', msg => logs.push(msg.text()));

  // Wait a bit more to capture all logs
  await page.waitForTimeout(1000);

  console.log('Browser console logs:');
  logs.forEach(log => console.log('  ', log));
});
