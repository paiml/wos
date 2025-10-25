// Debug test - check if multiple files are created
const { test, expect } = require('@playwright/test');

test('debug multiple file creation', async ({ page, context }) => {
  await context.clearCookies();
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

  // Create multiple files with SEPARATE commands
  await page.locator('#terminal-input').fill('touch file1.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  await page.locator('#terminal-input').fill('touch file2.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  await page.locator('#terminal-input').fill('touch file3.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Verify with ls command
  await page.locator('#terminal-input').fill('ls');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Check terminal output
  const terminalOutput = await page.locator('#terminal-output').textContent();
  console.log('\n=== TERMINAL OUTPUT ===');
  console.log(terminalOutput);
  console.log('=== END ===\n');

  // Check file list
  const fileItems = await page.locator('.file-item').count();
  console.log(`\nFile items in UI: ${fileItems}`);

  for (let i = 0; i < fileItems; i++) {
    const nameSpan = page.locator('.file-item').nth(i).locator('.file-item-name span');
    const name = await nameSpan.textContent();
    console.log(`  File ${i}: "${name}"`);
  }

  // Take screenshot
  await page.screenshot({ path: 'test-results/debug-multiple-files.png', fullPage: true });
});

test('debug multiple files with single touch command', async ({ page, context }) => {
  await context.clearCookies();
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

  // Create multiple files with SINGLE command (like the failing test)
  await page.locator('#terminal-input').fill('touch file1.txt file2.txt file3.txt');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(1500);

  // Verify with ls command
  await page.locator('#terminal-input').fill('ls');
  await page.locator('#terminal-input').press('Enter');
  await page.waitForTimeout(500);

  // Check terminal output
  const terminalOutput = await page.locator('#terminal-output').textContent();
  console.log('\n=== TERMINAL OUTPUT (single command) ===');
  console.log(terminalOutput);
  console.log('=== END ===\n');

  // Check file list
  const fileItems = await page.locator('.file-item').count();
  console.log(`\nFile items in UI: ${fileItems}`);

  for (let i = 0; i < fileItems; i++) {
    const nameSpan = page.locator('.file-item').nth(i).locator('.file-item-name span');
    const name = await nameSpan.textContent();
    console.log(`  File ${i}: "${name}"`);
  }

  // Take screenshot
  await page.screenshot({ path: 'test-results/debug-multiple-files-single-command.png', fullPage: true });
});
