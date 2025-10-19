import { test, expect } from '@playwright/test';

test.describe('Shell Scripts E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    await page.waitForSelector('#terminal-input', { timeout: 10000 });
    await page.waitForFunction(() => {
      const statusText = document.getElementById('status')?.textContent || '';
      return statusText === 'Ready';
    });
  });

  test('should create script in Vim, save, and execute with bash', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Open vim to create a script
    await input.fill('vim test_script.sh');
    await input.press('Enter');

    // Wait for vim modal
    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    // Enter INSERT mode
    await page.keyboard.press('i');

    // Type the script content
    await page.keyboard.type('#!/bin/bash');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Hello from script"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Script executed successfully"');

    // Exit INSERT mode
    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);

    // Save and quit
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');

    // Wait for vim to close
    await page.waitForSelector('.vim-modal', { state: 'hidden' });

    // Execute the script with bash
    await input.fill('bash test_script.sh');
    await input.press('Enter');

    // Verify output
    const outputText = await output.textContent();
    expect(outputText).toContain('Hello from script');
    expect(outputText).toContain('Script executed successfully');
  });

  test('should handle variable assignment and expansion', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Create a script with variables
    await input.fill('vim vars.sh');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    await page.keyboard.press('i');
    await page.keyboard.type('#!/bin/bash');
    await page.keyboard.press('Enter');
    await page.keyboard.type('NAME="World"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('COUNT=42');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Hello $NAME"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Count: $COUNT"');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'hidden' });

    // Execute the script
    await input.fill('bash vars.sh');
    await input.press('Enter');

    // Verify variable expansion
    const outputText = await output.textContent();
    expect(outputText).toContain('Hello World');
    expect(outputText).toContain('Count: 42');
  });

  test('should demonstrate source vs bash scope differences', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Create a script that sets a variable
    await input.fill('vim setvar.sh');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    await page.keyboard.press('i');
    await page.keyboard.type('#!/bin/bash');
    await page.keyboard.press('Enter');
    await page.keyboard.type('TESTVAR="sourced value"');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'hidden' });

    // Test with source (should set variable in current shell)
    await input.fill('source setvar.sh');
    await input.press('Enter');
    await page.waitForTimeout(200);

    await input.fill('echo $TESTVAR');
    await input.press('Enter');
    await page.waitForTimeout(200);

    let outputText = await output.textContent();
    expect(outputText).toContain('sourced value');

    // Clear the variable
    await input.fill('unset TESTVAR');
    await input.press('Enter');
    await page.waitForTimeout(200);

    // Test with bash (should NOT set variable in current shell)
    await input.fill('bash setvar.sh');
    await input.press('Enter');
    await page.waitForTimeout(200);

    await input.fill('echo $TESTVAR');
    await input.press('Enter');
    await page.waitForTimeout(200);

    outputText = await output.textContent();
    // The last echo $TESTVAR should be empty since bash runs in a subshell
    // The output should contain the first "sourced value" but after unset, it should be empty
    // Check that the output ends with the unset variable (empty echo output)
    const lines = outputText.split('\n').filter(line => line.trim() !== '');
    const lastCommand = lines[lines.length - 1];
    // Last command should be "echo $TESTVAR" with no output following it
    expect(lastCommand).toContain('echo $TESTVAR');
  });

  test('should execute script with ./script.sh syntax', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Create a script
    await input.fill('vim executable.sh');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    await page.keyboard.press('i');
    await page.keyboard.type('#!/bin/bash');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Executed with ./ syntax"');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'hidden' });

    // Execute with ./ syntax
    await input.fill('./executable.sh');
    await input.press('Enter');

    // Verify output
    const outputText = await output.textContent();
    expect(outputText).toContain('Executed with ./ syntax');
  });

  test('should handle script with multiple commands', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Create a multi-command script
    await input.fill('vim multi.sh');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    await page.keyboard.press('i');
    await page.keyboard.type('#!/bin/bash');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Command 1"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Command 2"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Command 3"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('VAR=test');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Variable: $VAR"');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'hidden' });

    // Execute the script
    await input.fill('bash multi.sh');
    await input.press('Enter');

    // Verify all commands executed
    const outputText = await output.textContent();
    expect(outputText).toContain('Command 1');
    expect(outputText).toContain('Command 2');
    expect(outputText).toContain('Command 3');
    expect(outputText).toContain('Variable: test');
  });

  test('should handle script error display', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Create a script with an invalid command
    await input.fill('vim error.sh');
    await input.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'visible' });
    await page.waitForTimeout(100);

    await page.keyboard.press('i');
    await page.keyboard.type('#!/bin/bash');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "Before error"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('invalid_command_xyz');
    await page.keyboard.press('Enter');
    await page.keyboard.type('echo "After error"');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
    await page.keyboard.type(':wq');
    await page.keyboard.press('Enter');

    await page.waitForSelector('.vim-modal', { state: 'hidden' });

    // Execute the script
    await input.fill('bash error.sh');
    await input.press('Enter');

    // Verify error handling
    const outputText = await output.textContent();
    expect(outputText).toContain('Before error');
    // Should show some kind of error message for invalid command
    expect(outputText).toMatch(/command not found|unknown command|error/i);
  });

  test('should display script not found error', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const output = page.locator('#terminal-output');

    // Try to execute a non-existent script with bash
    await input.fill('bash nonexistent.sh');
    await input.press('Enter');

    let outputText = await output.textContent();
    expect(outputText).toMatch(/script not found|not found|no such file/i);

    // Try with ./ syntax
    await input.fill('./another_nonexistent.sh');
    await input.press('Enter');

    outputText = await output.textContent();
    expect(outputText).toMatch(/script not found|not found|no such file/i);

    // Try with source
    await input.fill('source missing.sh');
    await input.press('Enter');

    outputText = await output.textContent();
    expect(outputText).toMatch(/script not found|not found|no such file/i);
  });
});
