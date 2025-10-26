const { test, expect } = require('@playwright/test');

test.describe('Debug if statement execution', () => {
  let consoleLogs = [];

  test.beforeEach(async ({ page }) => {
    // Capture all console messages
    page.on('console', msg => {
      const text = msg.text();
      consoleLogs.push(text);
      console.log(`[BROWSER CONSOLE] ${text}`);
    });

    await page.goto('http://localhost:8000/');
    await page.waitForSelector('#terminal', { state: 'visible', timeout: 10000 });
    await page.waitForTimeout(1000); // Wait for WASM to initialize
  });

  test('debug if-then-fi basic structure', async ({ page }) => {
    consoleLogs = []; // Reset

    // Type command to create script
    await page.locator('#terminal-input').fill('echo "#!/bin/bash" > /tmp/debug.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    await page.locator('#terminal-input').fill('echo "if true; then" >> /tmp/debug.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    await page.locator('#terminal-input').fill('echo "  echo success" >> /tmp/debug.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    await page.locator('#terminal-input').fill('echo "fi" >> /tmp/debug.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    // Show file contents
    await page.locator('#terminal-input').fill('cat /tmp/debug.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    // Execute the script
    consoleLogs = []; // Reset before execution
    await page.locator('#terminal-input').fill('bash /tmp/debug.sh');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(2000); // Wait longer for execution

    // Print captured console logs
    console.log('\n=== CAPTURED CONSOLE LOGS ===');
    consoleLogs.forEach((log, idx) => {
      console.log(`${idx}: ${log}`);
    });
    console.log('=== END CONSOLE LOGS ===\n');

    // Get terminal output
    const terminalOutput = await page.locator('#terminal-output').textContent();
    console.log('Terminal output:', terminalOutput);

    // Find DEBUG logs
    const debugLogs = consoleLogs.filter(log => log.includes('DEBUG'));
    console.log(`\nFound ${debugLogs.length} DEBUG messages`);
    debugLogs.forEach(log => console.log(log));
  });
});
