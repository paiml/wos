const { test, expect } = require('@playwright/test');

test.describe('Debug CSS class assignment', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:8000/');
    await page.waitForSelector('#terminal', { state: 'visible', timeout: 10000 });
    await page.waitForTimeout(1000); // Wait for WASM to initialize
  });

  test('check if bash output has terminal-line output classes', async ({ page }) => {
    // Execute the exact command from the test
    await page.locator('#terminal-input').fill('echo "#!/bin/bash\\nif true; then\\n  echo success\\nfi" > /tmp/exact_test.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    // Execute bash
    await page.locator('#terminal-input').fill('bash /tmp/exact_test.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(1000);

    // Check all terminal-line elements
    const allLines = await page.locator('.terminal-line').count();
    console.log(`Total terminal-line elements: ${allLines}`);

    // Check terminal-line.output elements
    const outputLines = await page.locator('.terminal-line.output').count();
    console.log(`terminal-line.output elements: ${outputLines}`);

    // Get last few lines
    for (let i = Math.max(0, allLines - 5); i < allLines; i++) {
      const line = page.locator('.terminal-line').nth(i);
      const text = await line.textContent();
      const classes = await line.getAttribute('class');
      console.log(`Line ${i}: classes="${classes}", text="${text}"`);
    }

    // Check last output line specifically
    if (outputLines > 0) {
      const lastOutput = page.locator('.terminal-line.output').nth(outputLines - 1);
      const lastText = await lastOutput.textContent();
      console.log(`Last .terminal-line.output text: "${lastText}"`);
    } else {
      console.log('NO .terminal-line.output elements found!');
    }

    // Also check if "success" appears ANYWHERE in terminal
    const terminalText = await page.locator('#terminal-output').textContent();
    console.log(`"success" in terminal: ${terminalText.includes('success')}`);
  });
});
