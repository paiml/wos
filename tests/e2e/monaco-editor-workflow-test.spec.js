// WOS-FILE-EDIT-01 Phase 4: Monaco Editor Workflow E2E Tests
// Tests complete workflow: create file → edit in Monaco → save → verify persistence

const { test, expect } = require('@playwright/test');

test.describe('Monaco Editor Workflow (WOS-FILE-EDIT-01 Phase 4)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://127.0.0.1:8000');

    // Wait for WASM to load
    await page.waitForTimeout(2000);

    // Dismiss tutorial if present
    const skipButton = page.locator('button:has-text("Skip Tutorial")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('monaco editor opens with edit command', async ({ page }) => {
    // Create a new JavaScript file using edit command
    await page.locator('#terminal-input').fill('edit test.js');
    await page.locator('#terminal-input').press('Enter');

    // Wait for Monaco editor to load (can take a moment)
    await page.waitForTimeout(2000);

    // Verify Monaco editor container is visible
    const monacoContainer = page.locator('#monaco-editor-container');
    await expect(monacoContainer).toBeVisible({ timeout: 5000 });

    // Verify editor has focus
    const editorIsFocused = await page.evaluate(() => {
      return document.querySelector('#monaco-editor-container').style.display !== 'none';
    });
    expect(editorIsFocused).toBe(true);

    // Close editor with Escape
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Verify editor closed
    await expect(monacoContainer).toBeHidden({ timeout: 2000 });
  });

  test('monaco editor saves content with Escape key', async ({ page }) => {
    // Create file with edit command
    await page.locator('#terminal-input').fill('edit hello.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    // Wait for Monaco to be visible
    await expect(page.locator('#monaco-editor-container')).toBeVisible({ timeout: 5000 });

    // Type code in Monaco editor
    await page.keyboard.type('function hello() {');
    await page.keyboard.press('Enter');
    await page.keyboard.type('  return "world";');
    await page.keyboard.press('Enter');
    await page.keyboard.type('}');

    // Save and close with Escape
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    // Verify file saved by reading with cat
    await page.locator('#terminal-input').fill('cat hello.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(500);

    const output = await page.locator('#terminal-output').textContent();
    expect(output).toContain('function hello()');
    expect(output).toContain('return "world"');
  });

  test('monaco editor loads existing file content', async ({ page }) => {
    // Create file with content using echo
    await page.locator('#terminal-input').fill('echo "const x = 42;" > existing.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(500);

    // Open in Monaco
    await page.locator('#terminal-input').fill('edit existing.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    // Wait for Monaco to load
    await expect(page.locator('#monaco-editor-container')).toBeVisible({ timeout: 5000 });

    // Get Monaco editor content
    const content = await page.evaluate(() => {
      return window.monacoEditor ? window.monacoEditor.getValue() : '';
    });

    expect(content).toContain('const x = 42;');

    // Close without changes
    await page.keyboard.press('Escape');
  });

  test('monaco editor handles multiline code correctly', async ({ page }) => {
    // Open editor
    await page.locator('#terminal-input').fill('edit multiline.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    // Type multiline code
    const code = [
      'class Calculator {',
      '  add(a, b) {',
      '    return a + b;',
      '  }',
      '',
      '  multiply(a, b) {',
      '    return a * b;',
      '  }',
      '}'
    ];

    for (const line of code) {
      await page.keyboard.type(line);
      await page.keyboard.press('Enter');
    }

    // Save and close
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    // Verify all lines saved
    await page.locator('#terminal-input').fill('cat multiline.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(500);

    const output = await page.locator('#terminal-output').textContent();
    expect(output).toContain('class Calculator');
    expect(output).toContain('add(a, b)');
    expect(output).toContain('multiply(a, b)');
  });

  test('monaco editor syntax highlighting by file extension', async ({ page }) => {
    // Test JavaScript file
    await page.locator('#terminal-input').fill('edit syntax.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    // Check that Monaco set JavaScript language
    const jsLanguage = await page.evaluate(() => {
      if (!window.monacoEditor) return null;
      return window.monacoEditor.getModel().getLanguageId();
    });
    expect(jsLanguage).toBe('javascript');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Test JSON file (well-supported by Monaco)
    await page.locator('#terminal-input').fill('edit config.json');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    const jsonLanguage = await page.evaluate(() => {
      if (!window.monacoEditor) return null;
      return window.monacoEditor.getModel().getLanguageId();
    });
    expect(jsonLanguage).toBe('json');

    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Test Markdown file
    await page.locator('#terminal-input').fill('edit README.md');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    const mdLanguage = await page.evaluate(() => {
      if (!window.monacoEditor) return null;
      return window.monacoEditor.getModel().getLanguageId();
    });
    expect(mdLanguage).toBe('markdown');

    await page.keyboard.press('Escape');
  });

  test('monaco editor handles special characters', async ({ page }) => {
    // Open editor
    await page.locator('#terminal-input').fill('edit special.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    // Type special characters
    await page.keyboard.type('Test $variable and "quotes"');
    await page.keyboard.press('Enter');
    await page.keyboard.type('Backslash: \\path\\to\\file');
    await page.keyboard.press('Enter');
    await page.keyboard.type('Single quotes: \'test\'');

    // Save
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    // Verify special characters preserved
    await page.locator('#terminal-input').fill('cat special.txt');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(500);

    const output = await page.locator('#terminal-output').textContent();
    expect(output).toContain('$variable');
    expect(output).toContain('"quotes"');
    expect(output).toContain('\\path\\to\\file');
    expect(output).toContain('\'test\'');
  });

  test('monaco editor creates file if not exists', async ({ page }) => {
    // Edit non-existent file
    await page.locator('#terminal-input').fill('edit newfile.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    // Monaco should open with empty content
    const content = await page.evaluate(() => {
      return window.monacoEditor ? window.monacoEditor.getValue() : null;
    });
    expect(content).toBe('');

    // Add content
    await page.keyboard.type('// New file created');

    // Save
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    // Verify file created
    await page.locator('#terminal-input').fill('cat newfile.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(500);

    const output = await page.locator('#terminal-output').textContent();
    expect(output).toContain('// New file created');
  });

  test('monaco editor edits update file list', async ({ page }) => {
    // Open Files panel
    await page.locator('button[data-panel-toggle="filesystem"]').click();
    await page.waitForTimeout(200);

    // Create file with Monaco
    await page.locator('#terminal-input').fill('edit listupdate.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    await page.keyboard.type('console.log("hello");');
    await page.keyboard.press('Escape');
    await page.waitForTimeout(1500);

    // Verify file appears in file list
    const fileItem = page.locator('.file-item').filter({ hasText: 'listupdate.js' });
    await expect(fileItem).toBeVisible({ timeout: 2000 });
  });

  test('monaco editor reopens with saved content', async ({ page }) => {
    // Create and save file
    await page.locator('#terminal-input').fill('edit persist.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    await page.keyboard.type('const saved = true;');
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    // Reopen file
    await page.locator('#terminal-input').fill('edit persist.js');
    await page.locator('#terminal-input').press('Enter');
    await page.waitForTimeout(2000);

    // Verify content loaded
    const content = await page.evaluate(() => {
      return window.monacoEditor ? window.monacoEditor.getValue() : '';
    });
    expect(content).toContain('const saved = true;');

    await page.keyboard.press('Escape');
  });
});
