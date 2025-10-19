import { test, expect } from '@playwright/test';

test.describe('WOS Bash Variable Support Debug', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    await page.waitForSelector('#terminal-output', { state: 'visible' });
  });

  test('should support variable assignment in terminal', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Clear any existing output
    const clearButton = page.locator('button:has-text("Clear")');
    await clearButton.click();

    // Test 1: Direct variable assignment and echo
    await input.fill('NAME="World"');
    await input.press('Enter');
    await page.waitForTimeout(100);

    await input.fill('echo "Hello $NAME"');
    await input.press('Enter');
    await page.waitForTimeout(300);

    const outputText = await output.textContent();
    console.log('Output after variable test:', outputText);

    // Check if variable expansion works
    if (outputText?.includes('Hello World')) {
      console.log('✅ Variables ARE supported in WOS bash');
    } else if (outputText?.includes('Hello ') || outputText?.includes('Hello$NAME')) {
      console.log('❌ Variables are NOT supported in WOS bash');
      console.log('   Variable was not expanded correctly');
    }

    // This test is diagnostic - we want to see what happens
    // Don't fail the test, just log results
    expect(outputText).toBeTruthy();
  });

  test('should show how cat reads file content', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Clear output
    const clearButton = page.locator('button:has-text("Clear")');
    await clearButton.click();

    // Create a file with vim containing a variable assignment
    await input.fill('vim test.sh');
    await input.press('Enter');
    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    await page.keyboard.press('i');
    await page.keyboard.type('#!/bin/bash');
    await page.keyboard.press('Enter');
    await page.keyboard.type('NAME="TestValue"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Value: $NAME"');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');
    await page.waitForSelector('.vim-modal', { state: 'hidden' });
    await page.waitForTimeout(200);

    // Now cat the file to see what was actually written
    await input.fill('cat test.sh');
    await input.press('Enter');
    await page.waitForTimeout(300);

    const outputText = await output.textContent();
    console.log('File content from cat:', outputText);

    // Look for what was actually written
    if (outputText?.includes('NAME="TestValue"')) {
      console.log('✅ File contains proper quotes: NAME="TestValue"');
    } else if (outputText?.includes('NAME=\\"TestValue\\"')) {
      console.log('❌ File has escaped quotes: NAME=\\"TestValue\\"');
    } else if (outputText?.includes('NAMETestValue')) {
      console.log('❌ File has no quotes at all');
    }

    expect(outputText).toBeTruthy();
  });

  test('should compare cat vs bash execution output', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Clear output
    const clearButton = page.locator('button:has-text("Clear")');
    await clearButton.click();

    // Create script
    await input.fill('vim debug.sh');
    await input.press('Enter');
    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    await page.keyboard.press('i');
    await page.keyboard.type('#!/bin/bash');
    await page.keyboard.press('Enter');
    await page.keyboard.type('VALUE="Test"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Output: $VALUE"');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');
    await page.waitForSelector('.vim-modal', { state: 'hidden' });
    await page.waitForTimeout(200);

    // Cat the file
    await clearButton.click();
    await input.fill('cat debug.sh');
    await input.press('Enter');
    await page.waitForTimeout(300);

    let catOutput = await output.textContent();
    console.log('=== CAT OUTPUT ===');
    console.log(catOutput);

    // Execute with bash
    await clearButton.click();
    await input.fill('bash debug.sh');
    await input.press('Enter');
    await page.waitForTimeout(300);

    let bashOutput = await output.textContent();
    console.log('=== BASH OUTPUT ===');
    console.log(bashOutput);

    expect(catOutput).toBeTruthy();
    expect(bashOutput).toBeTruthy();
  });
});
