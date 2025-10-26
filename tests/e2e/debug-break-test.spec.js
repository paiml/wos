const { test } = require('@playwright/test');

async function executeCommand(page, command) {
  const input = page.locator('#terminal-input');
  await input.fill(command);
  await input.press('Enter');
  await page.waitForTimeout(300);
}

test('Debug break test script', async ({ page }) => {
  await page.goto('http://127.0.0.1:8000');
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForTimeout(2000);

  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Use the EXACT command from the passing "while loop with break" test
  await executeCommand(page, 'echo "#!/bin/bash\\\\nCOUNT=0\\\\nwhile true; do\\\\n  echo \\\\$COUNT\\\\n  COUNT=\\\\$((COUNT + 1))\\\\n  if [ \\\\$COUNT -ge 2 ]; then\\\\n    break\\\\n  fi\\\\ndone" > /tmp/while_break.sh');
  await page.waitForTimeout(200);

  await executeCommand(page, 'cat /tmp/while_break.sh');
  await page.waitForTimeout(500);

  const outputs = page.locator('.terminal-line.output');
  const count = await outputs.count();
  console.log(`File contents:`);

  for (let i = 0; i < count; i++) {
    const text = await outputs.nth(i).textContent();
    console.log(text);
  }

  // Now try to run it
  await executeCommand(page, 'bash /tmp/while_break.sh');
  await page.waitForTimeout(1000);

  const count2 = await outputs.count();
  console.log(`\nBash execution output:`);
  for (let i = count; i < count2; i++) {
    const text = await outputs.nth(i).textContent();
    console.log(text);
  }
});
