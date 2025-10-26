const { test, expect } = require('@playwright/test');

test.describe('Debug echo newline handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:8000/');
    await page.waitForSelector('#terminal', { state: 'visible', timeout: 10000 });
    await page.waitForTimeout(1000); // Wait for WASM to initialize
  });

  test('test exact command from failing test', async ({ page }) => {
    // Use EXACT command from bash-control-structures-test.spec.js
    await page.locator('#terminal-input').fill('echo "#!/bin/bash\\nif true; then\\n  echo success\\nfi" > /tmp/exact_test.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    // Show file contents
    await page.locator('#terminal-input').fill('cat /tmp/exact_test.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    // Get terminal output to see what cat shows
    const terminalOutput = await page.locator('#terminal-output').textContent();
    console.log('Terminal output after cat:', terminalOutput);

    // Check if file has actual newlines or literal \n
    const catOutput = terminalOutput.split('cat /tmp/exact_test.sh')[1] || '';
    console.log('Cat output:', catOutput);

    // Execute the script
    await page.locator('#terminal-input').fill('bash /tmp/exact_test.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(1000);

    // Get final output
    const finalOutput = await page.locator('#terminal-output').textContent();
    console.log('Final output:', finalOutput);

    // Find the output after bash command
    const bashOutput = finalOutput.split('bash /tmp/exact_test.sh')[1] || '';
    console.log('Bash output:', bashOutput);
  });
});
