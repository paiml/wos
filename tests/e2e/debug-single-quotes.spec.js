const { test } = require('@playwright/test');

async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(300);
}

test('Test single quotes', async ({ page }) => {
  // Force hard reload to bypass cache
  await page.goto('http://127.0.0.1:8000', { waitUntil: 'networkidle' });
  await page.evaluate(() => {
    localStorage.clear();
    // Force reload from server, not cache
    window.location.reload(true);
  });
  await page.waitForTimeout(3000);

  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Test with single quotes
  await executeCommand(page, "echo '#!/bin/bash' > /tmp/test.sh");
  await executeCommand(page, "echo 'COUNT=0' >> /tmp/test.sh");
  await executeCommand(page, "echo 'while [ $COUNT -lt 3 ]; do' >> /tmp/test.sh");
  await executeCommand(page, "echo '  echo $COUNT' >> /tmp/test.sh");
  await executeCommand(page, "echo '  COUNT=$((COUNT + 1))' >> /tmp/test.sh");
  await executeCommand(page, "echo 'done' >> /tmp/test.sh");
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
