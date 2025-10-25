// Debug test - check if file deletion updates the list
const { test, expect } = require('@playwright/test');

test('debug file deletion', async ({ page, context }) => {
  await context.clearCookies();

  // Capture console messages
  const consoleMsgs = [];
  page.on('console', msg => {
    consoleMsgs.push(`[${msg.type()}] ${msg.text()}`);
  });

  await page.goto('http://127.0.0.1:8000/?trace=DEBUG&categories=FILE', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);

  // Dismiss tutorial
  const skipButton = page.locator('button:has-text("Skip Tutorial")');
  if (await skipButton.isVisible()) {
    await skipButton.click();
    await page.waitForTimeout(500);
  }

  // Open Files panel
  await page.locator('button[data-panel-toggle="filesystem"]').click();
  await page.waitForTimeout(500);

  // Create file
  await page.locator('#terminal-input').fill('touch deleteme.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Verify file exists with ls
  await page.locator('#terminal-input').fill('ls');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  let terminalOutput = await page.locator('#terminal-output').textContent();
  console.log('\n=== AFTER TOUCH ===');
  console.log(terminalOutput.slice(-200)); // Last 200 chars
  console.log('=== END ===\n');

  // Check file list
  let fileItems = await page.locator('.file-item').count();
  console.log(`File items BEFORE deletion: ${fileItems}`);

  for (let i = 0; i < fileItems; i++) {
    const nameSpan = page.locator('.file-item').nth(i).locator('.file-item-name span');
    const name = await nameSpan.textContent();
    console.log(`  File ${i}: "${name}"`);
  }

  // Delete file
  await page.locator('#terminal-input').fill('rm deleteme.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Verify file deleted with ls
  await page.locator('#terminal-input').fill('ls');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  terminalOutput = await page.locator('#terminal-output').textContent();
  console.log('\n=== AFTER RM ===');
  console.log(terminalOutput.slice(-200)); // Last 200 chars
  console.log('=== END ===\n');

  // Check file list
  fileItems = await page.locator('.file-item').count();
  console.log(`File items AFTER deletion: ${fileItems}`);

  for (let i = 0; i < fileItems; i++) {
    const nameSpan = page.locator('.file-item').nth(i).locator('.file-item-name span');
    const name = await nameSpan.textContent();
    console.log(`  File ${i}: "${name}"`);
  }

  // Take screenshot
  await page.screenshot({ path: 'test-results/debug-deletion.png', fullPage: true });

  // Print all console messages with FILE category
  console.log('\n=== CONSOLE MESSAGES (FILE category) ===');
  consoleMsgs.filter(msg => msg.includes('[FILE]')).forEach(msg => console.log(msg));
  console.log('=== END CONSOLE ===\n');
});
