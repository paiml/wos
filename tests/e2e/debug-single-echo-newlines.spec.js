const { test } = require('@playwright/test');

async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(300);
}

test('Test single echo with newlines', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000');
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForTimeout(2000);

  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Use the EXACT pattern from the working "while loop with break" test
  await executeCommand(page, 'echo "#!/bin/bash\\\\nCOUNT=0\\\\nwhile [ \\\\$COUNT -lt 3 ]; do\\\\n  echo \\\\$COUNT\\\\n  COUNT=\\\\$((COUNT + 1))\\\\ndone" > /tmp/test.sh');
  await page.waitForTimeout(200);

  await executeCommand(page, 'cat /tmp/test.sh');
  await page.waitForTimeout(500);

  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  console.log(`Total lines: ${count}`);

  for (let i = 0; i < count; i++) {
    const text = await outputs.nth(i).textContent();
    console.log(`Output ${i}: "${text}"`);
  }

  // Now try to run it
  await executeCommand(page, 'bash /tmp/test.sh');
  await page.waitForTimeout(1000);

  const count2 = await outputs.count();
  console.log(`\nAfter bash execution:`);
  for (let i = count; i < count2; i++) {
    const text = await outputs.nth(i).textContent();
    console.log(`Output ${i}: "${text}"`);
  }
});
