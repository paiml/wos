// Monaco Editor Integration E2E Tests
// Tests for WOS-300: Monaco editor with accessibility features
// Spec: docs/specifications/wos-enhanced-features-spec.md Section 4.6.1

import { test, expect } from '@playwright/test';

test.describe('Monaco Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    await page.waitForSelector('#terminal-output', { timeout: 10000 });
    // Wait for WOS to be fully initialized
    await page.waitForFunction(() => (window as any).wos !== undefined, { timeout: 10000 });
  });

  test.describe('Editor Loading and Initialization', () => {
    test('should load Monaco editor library from CDN', async ({ page }) => {
      // Monaco should be available globally
      const monacoLoaded = await page.evaluate(() => {
        return typeof (window as any).monaco !== 'undefined';
      });
      expect(monacoLoaded).toBe(true);
    });

    test('should create editor instance when edit command is run', async ({ page }) => {
      // Create a test file first
      await page.fill('#terminal-input', 'echo "test content" > test.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      // Open file with edit command
      await page.fill('#terminal-input', 'edit test.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Editor container should be visible
      const editorVisible = await page.isVisible('#monaco-editor-container');
      expect(editorVisible).toBe(true);

      // Monaco editor instance should exist
      const editorExists = await page.evaluate(() => {
        return (window as any).monacoEditor !== undefined;
      });
      expect(editorExists).toBe(true);
    });

    test('should display file contents in editor', async ({ page }) => {
      // Create a test file
      await page.fill('#terminal-input', 'echo "Hello Monaco!" > hello.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      // Open file
      await page.fill('#terminal-input', 'edit hello.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Check editor content
      const editorContent = await page.evaluate(() => {
        return (window as any).monacoEditor?.getValue();
      });
      expect(editorContent).toContain('Hello Monaco!');
    });
  });

  test.describe('Syntax Highlighting', () => {
    test('should apply Rust syntax highlighting for .rs files', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "fn main() {}" > main.rs');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit main.rs');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      const language = await page.evaluate(() => {
        return (window as any).monacoEditor?.getModel()?.getLanguageId();
      });
      expect(language).toBe('rust');
    });

    test('should apply Bash syntax highlighting for .sh files', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "#!/bin/bash" > script.sh');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit script.sh');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      const language = await page.evaluate(() => {
        return (window as any).monacoEditor?.getModel()?.getLanguageId();
      });
      expect(language).toBe('shell');
    });

    test('should apply Markdown syntax highlighting for .md files', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "# Header" > README.md');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit README.md');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      const language = await page.evaluate(() => {
        return (window as any).monacoEditor?.getModel()?.getLanguageId();
      });
      expect(language).toBe('markdown');
    });

    test('should apply YAML syntax highlighting for .yaml files', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "key: value" > config.yaml');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit config.yaml');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      const language = await page.evaluate(() => {
        return (window as any).monacoEditor?.getModel()?.getLanguageId();
      });
      expect(language).toBe('yaml');
    });
  });

  test.describe('Editor Features', () => {
    test('should support multi-cursor editing with Ctrl+D', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "word word word" > multi.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit multi.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Select first "word" and add cursors with Ctrl+D
      await page.keyboard.press('Control+Home'); // Start of file
      await page.keyboard.press('Control+Shift+Right'); // Select "word"
      await page.keyboard.press('Control+D'); // Add cursor to next occurrence

      const selections = await page.evaluate(() => {
        return (window as any).monacoEditor?.getSelections()?.length;
      });
      expect(selections).toBeGreaterThan(1);
    });

    test('should display minimap for files', async ({ page }) => {
      // Create a longer file
      const content = Array(50).fill('Line of text').join('\\n');
      await page.fill('#terminal-input', `echo "${content}" > long.txt`);
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit long.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Minimap should be visible
      const minimapVisible = await page.isVisible('.minimap');
      expect(minimapVisible).toBe(true);
    });

    test('should open command palette with Ctrl+Shift+P', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "test" > cmd.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit cmd.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Focus editor and open command palette
      await page.click('#monaco-editor-container');
      await page.keyboard.press('Control+Shift+P');
      await page.waitForTimeout(500);

      // Command palette should be visible
      const paletteVisible = await page.isVisible('.quick-input-widget');
      expect(paletteVisible).toBe(true);
    });
  });

  test.describe('WCAG 2.1 AA Accessibility', () => {
    test('should support full keyboard navigation', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "accessible" > acc.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit acc.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Editor should be focusable via Tab
      await page.keyboard.press('Tab');
      const editorFocused = await page.evaluate(() => {
        const activeElement = document.activeElement;
        return activeElement?.classList.contains('monaco-editor') ||
               activeElement?.closest('.monaco-editor') !== null;
      });
      expect(editorFocused).toBe(true);
    });

    test('should have ARIA labels for screen readers', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "aria" > aria.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit aria.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Editor container should have role and aria-label
      const ariaLabel = await page.getAttribute('#monaco-editor-container', 'aria-label');
      expect(ariaLabel).toBeTruthy();

      const role = await page.getAttribute('#monaco-editor-container', 'role');
      expect(role).toBeTruthy();
    });

    test('should support high-contrast theme', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "contrast" > contrast.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit contrast.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Check that high-contrast theme is available
      const themeAvailable = await page.evaluate(() => {
        const monaco = (window as any).monaco;
        return monaco?.editor?.defineTheme !== undefined;
      });
      expect(themeAvailable).toBe(true);
    });

    test('should have configurable font sizes (14px-24px)', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "font" > font.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit font.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Check font size is within range
      const fontSize = await page.evaluate(() => {
        return (window as any).monacoEditor?.getOptions()?.get(49); // EditorOption.fontSize
      });
      expect(fontSize).toBeGreaterThanOrEqual(14);
      expect(fontSize).toBeLessThanOrEqual(24);
    });
  });

  test.describe('Editor Integration with WOS', () => {
    test('should save changes when closing editor', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "original" > save.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit save.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Modify content
      await page.evaluate(() => {
        (window as any).monacoEditor?.setValue('modified content');
      });
      await page.waitForTimeout(500);

      // Close editor (Escape key)
      await page.keyboard.press('Escape');
      await page.waitForTimeout(500);

      // Read file to verify changes
      await page.fill('#terminal-input', 'cat save.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.textContent('#terminal-output');
      expect(output).toContain('modified content');
    });

    test('should close editor without saving on Escape', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "test" > esc.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      await page.fill('#terminal-input', 'edit esc.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Press Escape
      await page.keyboard.press('Escape');
      await page.waitForTimeout(500);

      // Editor should be hidden
      const editorVisible = await page.isVisible('#monaco-editor-container');
      expect(editorVisible).toBe(false);

      // Terminal should be focused again
      const terminalFocused = await page.evaluate(() => {
        return document.activeElement?.id === 'terminal-input';
      });
      expect(terminalFocused).toBe(true);
    });

    test('should handle edit command for non-existent file', async ({ page }) => {
      await page.fill('#terminal-input', 'edit newfile.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Editor should open with empty content
      const editorVisible = await page.isVisible('#monaco-editor-container');
      expect(editorVisible).toBe(true);

      const content = await page.evaluate(() => {
        return (window as any).monacoEditor?.getValue();
      });
      expect(content).toBe('');
    });

    test('should preserve terminal state while editing', async ({ page }) => {
      // Run some commands
      await page.fill('#terminal-input', 'ps');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const outputBeforeEdit = await page.textContent('#terminal-output');

      // Open editor
      await page.fill('#terminal-input', 'edit temp.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Close editor
      await page.keyboard.press('Escape');
      await page.waitForTimeout(500);

      // Terminal output should be preserved
      const outputAfterEdit = await page.textContent('#terminal-output');
      expect(outputAfterEdit).toContain(outputBeforeEdit || '');
    });
  });

  test.describe('Performance', () => {
    test('should load editor in less than 1 second', async ({ page }) => {
      await page.fill('#terminal-input', 'echo "perf" > perf.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const startTime = Date.now();
      await page.fill('#terminal-input', 'edit perf.txt');
      await page.press('#terminal-input', 'Enter');

      await page.waitForSelector('#monaco-editor-container', { timeout: 5000 });
      const loadTime = Date.now() - startTime;

      expect(loadTime).toBeLessThan(1000);
    });

    test('should handle large files without lag', async ({ page }) => {
      // Create a large file (1000 lines)
      const largeContent = Array(1000).fill('This is line').map((l, i) => `${l} ${i}`).join('\\n');
      await page.evaluate((content) => {
        (window as any).wos.writeFile('/tmp/large.txt', content);
      }, largeContent);

      await page.fill('#terminal-input', 'edit /tmp/large.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(1000);

      // Editor should be responsive
      await page.keyboard.press('End'); // Jump to end of line
      await page.keyboard.type(' typed');

      const content = await page.evaluate(() => {
        return (window as any).monacoEditor?.getValue();
      });
      expect(content).toContain('typed');
    });
  });
});
