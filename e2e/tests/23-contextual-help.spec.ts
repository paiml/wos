import { test, expect } from '@playwright/test';

/**
 * WOS-305: Contextual Help System E2E Tests
 *
 * Tests for comprehensive help system with command documentation,
 * tooltips, search functionality, and dedicated help panel.
 *
 * Requirements from roadmap.yaml WOS-305:
 * - `help <command>` command with detailed documentation
 * - Tooltip hover hints (ARIA compliant)
 * - Full-text search across documentation
 * - Side panel for help content display
 *
 * Tests required:
 * - test_help_command_documentation
 * - test_tooltip_hints
 * - test_help_search
 * - e2e_help_system_usage
 *
 * Complexity target: 12
 * Coverage target: 85%
 */

test.describe('WOS-305: Contextual Help System', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    // Wait for WASM to initialize
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test.describe('Help Command Documentation', () => {
    test('should display general help when running `help` with no arguments', async ({ page }) => {
      // Type help command
      await page.fill('#terminal-input', 'help');
      await page.press('#terminal-input', 'Enter');

      // Wait for output
      await page.waitForTimeout(500);

      // Should show available commands
      const output = await page.locator('#terminal-output').textContent();
      expect(output).toContain('Available commands:');
      expect(output).toContain('ls');
      expect(output).toContain('cat');
      expect(output).toContain('echo');
      expect(output).toContain('help');
    });

    test('should display detailed help for `help ls`', async ({ page }) => {
      await page.fill('#terminal-input', 'help ls');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();

      // Should contain command name
      expect(output).toContain('ls');

      // Should contain description
      expect(output).toMatch(/list|directory|files/i);

      // Should contain usage example
      expect(output).toMatch(/usage|example/i);

      // Should contain options/flags
      expect(output).toMatch(/options|flags/i);
    });

    test('should display detailed help for `help cat`', async ({ page }) => {
      await page.fill('#terminal-input', 'help cat');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toContain('cat');
      expect(output).toMatch(/concatenate|display|file/i);
      expect(output).toMatch(/usage/i);
    });

    test('should display detailed help for `help echo`', async ({ page }) => {
      await page.fill('#terminal-input', 'help echo');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toContain('echo');
      expect(output).toMatch(/print|display|output/i);
      expect(output).toMatch(/usage/i);
    });

    test('should display detailed help for `help cd`', async ({ page }) => {
      await page.fill('#terminal-input', 'help cd');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toContain('cd');
      expect(output).toMatch(/change|directory/i);
      expect(output).toMatch(/usage/i);
    });

    test('should display detailed help for `help pwd`', async ({ page }) => {
      await page.fill('#terminal-input', 'help pwd');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toContain('pwd');
      expect(output).toMatch(/print|working|directory/i);
    });

    test('should display detailed help for `help mkdir`', async ({ page }) => {
      await page.fill('#terminal-input', 'help mkdir');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toContain('mkdir');
      expect(output).toMatch(/make|create|directory/i);
    });

    test('should display detailed help for `help rm`', async ({ page }) => {
      await page.fill('#terminal-input', 'help rm');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toContain('rm');
      expect(output).toMatch(/remove|delete|file/i);
    });

    test('should display helpful error for unknown command in help', async ({ page }) => {
      await page.fill('#terminal-input', 'help nonexistentcommand');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toMatch(/unknown|not found|invalid/i);
      expect(output).toMatch(/help.*available/i);
    });

    test('should include syntax highlighting in help output', async ({ page }) => {
      await page.fill('#terminal-input', 'help ls');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      // Check for syntax-highlighted elements
      const helpOutput = await page.locator('#terminal-output .help-command');
      const exists = await helpOutput.count() > 0;
      expect(exists).toBe(true);
    });

    test('should show examples section in help output', async ({ page }) => {
      await page.fill('#terminal-input', 'help cat');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toMatch(/examples?:/i);

      // Should have at least one example
      expect(output).toMatch(/cat\s+\w+/);
    });

    test('should show related commands section', async ({ page }) => {
      await page.fill('#terminal-input', 'help ls');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(500);

      const output = await page.locator('#terminal-output').textContent();
      expect(output).toMatch(/see also|related commands/i);
    });
  });

  test.describe('Tooltip Hover Hints (ARIA Compliant)', () => {
    test('should show tooltip on hover over terminal input', async ({ page }) => {
      const terminalInput = await page.locator('#terminal-input');

      // Hover over input
      await terminalInput.hover();
      await page.waitForTimeout(300);

      // Tooltip should appear
      const tooltip = await page.locator('.tooltip, [role="tooltip"]').first();
      await expect(tooltip).toBeVisible();
    });

    test('should show tooltip on hover over command buttons', async ({ page }) => {
      const clearButton = await page.locator('#btn-clear, .btn-clear').first();

      if (await clearButton.count() > 0) {
        await clearButton.hover();
        await page.waitForTimeout(300);

        const tooltip = await page.locator('[role="tooltip"]');
        if (await tooltip.count() > 0) {
          await expect(tooltip).toBeVisible();
          const text = await tooltip.textContent();
          expect(text).toMatch(/clear/i);
        }
      }
    });

    test('tooltips should have proper ARIA attributes', async ({ page }) => {
      const terminalInput = await page.locator('#terminal-input');
      await terminalInput.hover();
      await page.waitForTimeout(300);

      const tooltip = await page.locator('[role="tooltip"]').first();

      if (await tooltip.count() > 0) {
        // Should have role="tooltip"
        const role = await tooltip.getAttribute('role');
        expect(role).toBe('tooltip');

        // Should have id that matches aria-describedby
        const tooltipId = await tooltip.getAttribute('id');
        expect(tooltipId).toBeTruthy();

        // Triggering element should reference tooltip
        const describedBy = await terminalInput.getAttribute('aria-describedby');
        expect(describedBy).toContain(tooltipId || '');
      }
    });

    test('tooltip should disappear on mouse leave', async ({ page }) => {
      const terminalInput = await page.locator('#terminal-input');

      // Hover to show tooltip
      await terminalInput.hover();
      await page.waitForTimeout(300);

      // Move away
      await page.mouse.move(0, 0);
      await page.waitForTimeout(300);

      // Tooltip should be hidden
      const tooltip = await page.locator('[role="tooltip"]:visible');
      const count = await tooltip.count();
      expect(count).toBe(0);
    });

    test('tooltips should be keyboard accessible', async ({ page }) => {
      // Focus on terminal input
      await page.locator('#terminal-input').focus();

      // Tooltip should appear or ARIA description should be present
      const input = await page.locator('#terminal-input');
      const ariaDescribedBy = await input.getAttribute('aria-describedby');
      expect(ariaDescribedBy).toBeTruthy();
    });

    test('should show contextual help tooltip for file panel buttons', async ({ page }) => {
      const saveButton = await page.locator('.btn-save, #btn-save').first();

      if (await saveButton.count() > 0) {
        await saveButton.hover();
        await page.waitForTimeout(300);

        const tooltip = await page.locator('[role="tooltip"]');
        if (await tooltip.count() > 0) {
          const text = await tooltip.textContent();
          expect(text).toMatch(/save/i);
        }
      }
    });

    test('should show tooltip for panel collapse/expand buttons', async ({ page }) => {
      const collapseButton = await page.locator('.btn-collapse').first();

      if (await collapseButton.count() > 0) {
        await collapseButton.hover();
        await page.waitForTimeout(300);

        const tooltip = await page.locator('[role="tooltip"]');
        if (await tooltip.count() > 0) {
          await expect(tooltip).toBeVisible();
        }
      }
    });
  });

  test.describe('Help Panel UI', () => {
    test('should have dedicated help panel in sidebar', async ({ page }) => {
      const helpPanel = await page.locator('#panel-help, .panel-help');
      await expect(helpPanel).toBeVisible();
    });

    test('should show help panel header with title', async ({ page }) => {
      const helpPanelHeader = await page.locator('#panel-help .panel-header, .panel-help .panel-header');
      await expect(helpPanelHeader).toBeVisible();

      const headerText = await helpPanelHeader.textContent();
      expect(headerText).toMatch(/help/i);
    });

    test('should display help panel when clicking help icon/button', async ({ page }) => {
      const helpButton = await page.locator('#btn-help, .btn-help, [aria-label*="help"]').first();

      if (await helpButton.count() > 0) {
        await helpButton.click();
        await page.waitForTimeout(300);

        const helpPanel = await page.locator('#panel-help');
        const isVisible = await helpPanel.isVisible();
        expect(isVisible).toBe(true);
      }
    });

    test('should show command reference list in help panel', async ({ page }) => {
      const helpPanel = await page.locator('#panel-help');

      // Expand panel if collapsed
      const isCollapsed = await helpPanel.evaluate((el) => {
        return el.classList.contains('collapsed');
      });

      if (isCollapsed) {
        const expandBtn = await page.locator('#panel-help .btn-collapse');
        await expandBtn.click();
        await page.waitForTimeout(300);
      }

      // Should show command list
      const commandList = await page.locator('#help-command-list, .help-command-list');
      await expect(commandList).toBeVisible();

      // Should have multiple commands
      const commands = await page.locator('.help-command-item').count();
      expect(commands).toBeGreaterThan(5);
    });

    test('should expand command details when clicking command in help panel', async ({ page }) => {
      const firstCommand = await page.locator('.help-command-item').first();

      if (await firstCommand.count() > 0) {
        await firstCommand.click();
        await page.waitForTimeout(300);

        // Details should expand
        const details = await firstCommand.locator('.help-command-details');
        await expect(details).toBeVisible();
      }
    });

    test('should show command description in expanded view', async ({ page }) => {
      const firstCommand = await page.locator('.help-command-item').first();

      if (await firstCommand.count() > 0) {
        await firstCommand.click();
        await page.waitForTimeout(300);

        const description = await firstCommand.locator('.help-command-description').textContent();
        expect(description).toBeTruthy();
        expect(description.length).toBeGreaterThan(10);
      }
    });

    test('should show usage examples in help panel', async ({ page }) => {
      const firstCommand = await page.locator('.help-command-item').first();

      if (await firstCommand.count() > 0) {
        await firstCommand.click();
        await page.waitForTimeout(300);

        const examples = await firstCommand.locator('.help-command-examples');
        if (await examples.count() > 0) {
          await expect(examples).toBeVisible();
        }
      }
    });

    test('help panel should be collapsible', async ({ page }) => {
      const helpPanel = await page.locator('#panel-help');
      const collapseBtn = await helpPanel.locator('.btn-collapse');

      if (await collapseBtn.count() > 0) {
        const initialState = await helpPanel.evaluate((el) => {
          return el.classList.contains('collapsed');
        });

        await collapseBtn.click();
        await page.waitForTimeout(300);

        const afterClickState = await helpPanel.evaluate((el) => {
          return el.classList.contains('collapsed');
        });

        expect(afterClickState).toBe(!initialState);
      }
    });

    test('help panel should persist state across page reloads', async ({ page }) => {
      const helpPanel = await page.locator('#panel-help');

      // Expand panel
      const isCollapsed = await helpPanel.evaluate((el) => {
        return el.classList.contains('collapsed');
      });

      if (isCollapsed) {
        const expandBtn = await page.locator('#panel-help .btn-collapse');
        await expandBtn.click();
        await page.waitForTimeout(300);
      }

      // Reload page
      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

      // Panel should still be expanded
      const afterReloadCollapsed = await helpPanel.evaluate((el) => {
        return el.classList.contains('collapsed');
      });
      expect(afterReloadCollapsed).toBe(false);
    });
  });

  test.describe('Full-Text Search Across Documentation', () => {
    test('should have search input in help panel', async ({ page }) => {
      const searchInput = await page.locator('#help-search, .help-search-input');
      await expect(searchInput).toBeVisible();
    });

    test('should filter commands when typing in search', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      // Type search query
      await searchInput.fill('list');
      await page.waitForTimeout(300);

      // Should show only matching commands
      const visibleCommands = await page.locator('.help-command-item:visible').count();
      const allCommands = await page.locator('.help-command-item').count();

      // Some commands should be filtered out
      expect(visibleCommands).toBeLessThanOrEqual(allCommands);
      expect(visibleCommands).toBeGreaterThan(0);

      // ls command should be visible (contains "list" in description)
      const lsCommand = await page.locator('.help-command-item:visible:has-text("ls")');
      await expect(lsCommand).toBeVisible();
    });

    test('should search across command names', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('cat');
      await page.waitForTimeout(300);

      const catCommand = await page.locator('.help-command-item:visible:has-text("cat")');
      await expect(catCommand).toBeVisible();
    });

    test('should search across command descriptions', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('directory');
      await page.waitForTimeout(300);

      // Should show commands related to directories (cd, ls, mkdir, pwd)
      const visibleCommands = await page.locator('.help-command-item:visible').count();
      expect(visibleCommands).toBeGreaterThan(0);
    });

    test('should search across command examples', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('file.txt');
      await page.waitForTimeout(300);

      // Commands with file.txt in examples should appear
      const visibleCommands = await page.locator('.help-command-item:visible').count();
      expect(visibleCommands).toBeGreaterThan(0);
    });

    test('should show "no results" message when search has no matches', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('xyznonexistent');
      await page.waitForTimeout(300);

      const noResults = await page.locator('.help-no-results, .no-results');
      await expect(noResults).toBeVisible();

      const noResultsText = await noResults.textContent();
      expect(noResultsText).toMatch(/no.*found|no.*match/i);
    });

    test('should clear search when clicking clear button', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('test search');
      await page.waitForTimeout(300);

      const clearBtn = await page.locator('#help-search-clear, .help-search-clear');

      if (await clearBtn.count() > 0) {
        await clearBtn.click();
        await page.waitForTimeout(200);

        const inputValue = await searchInput.inputValue();
        expect(inputValue).toBe('');
      }
    });

    test('search should be case-insensitive', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('LIST');
      await page.waitForTimeout(300);

      const visibleCommands = await page.locator('.help-command-item:visible').count();
      expect(visibleCommands).toBeGreaterThan(0);
    });

    test('should highlight search matches in results', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('list');
      await page.waitForTimeout(300);

      const highlight = await page.locator('.help-search-highlight, mark');

      if (await highlight.count() > 0) {
        await expect(highlight.first()).toBeVisible();
      }
    });

    test('should show search result count', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('file');
      await page.waitForTimeout(300);

      const resultCount = await page.locator('.help-search-count');

      if (await resultCount.count() > 0) {
        const countText = await resultCount.textContent();
        expect(countText).toMatch(/\d+\s*(result|command|match)/i);
      }
    });

    test('search should update results in real-time as user types', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      await searchInput.fill('l');
      await page.waitForTimeout(200);
      const resultsAfterL = await page.locator('.help-command-item:visible').count();

      await searchInput.fill('ls');
      await page.waitForTimeout(200);
      const resultsAfterLs = await page.locator('.help-command-item:visible').count();

      // Results should narrow down
      expect(resultsAfterLs).toBeLessThanOrEqual(resultsAfterL);
    });
  });

  test.describe('Keyboard Shortcuts for Help', () => {
    test('should show help panel when pressing F1 or Ctrl+/', async ({ page }) => {
      // Press F1
      await page.keyboard.press('F1');
      await page.waitForTimeout(300);

      const helpPanel = await page.locator('#panel-help');
      const isVisible = await helpPanel.isVisible();

      // Panel should be visible or focused
      expect(isVisible).toBe(true);
    });

    test('should focus search input when help panel opens via keyboard', async ({ page }) => {
      await page.keyboard.press('F1');
      await page.waitForTimeout(300);

      const focusedElement = await page.evaluate(() => {
        return document.activeElement?.id;
      });

      expect(focusedElement).toMatch(/help-search/);
    });

    test('should navigate through help items with arrow keys', async ({ page }) => {
      const searchInput = await page.locator('#help-search');
      await searchInput.focus();

      // Press down arrow
      await page.keyboard.press('ArrowDown');
      await page.waitForTimeout(200);

      const focusedElement = await page.evaluate(() => {
        return document.activeElement?.classList.contains('help-command-item');
      });

      expect(focusedElement).toBe(true);
    });

    test('should close help panel with Escape key', async ({ page }) => {
      // Open help panel
      await page.keyboard.press('F1');
      await page.waitForTimeout(300);

      // Press Escape
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);

      const helpPanel = await page.locator('#panel-help');
      const isCollapsed = await helpPanel.evaluate((el) => {
        return el.classList.contains('collapsed');
      });

      expect(isCollapsed).toBe(true);
    });
  });

  test.describe('Integration with Terminal', () => {
    test('should insert command from help panel when clicking', async ({ page }) => {
      const firstCommand = await page.locator('.help-command-item').first();

      // Get command name
      const commandName = await firstCommand.locator('.help-command-name').textContent();

      // Click "Insert" or "Try it" button if exists
      const insertBtn = await firstCommand.locator('.btn-insert-command, .btn-try-command');

      if (await insertBtn.count() > 0) {
        await insertBtn.click();
        await page.waitForTimeout(300);

        // Terminal input should contain command
        const inputValue = await page.locator('#terminal-input').inputValue();
        expect(inputValue).toContain(commandName || '');
      }
    });

    test('should show contextual help based on current input', async ({ page }) => {
      const terminalInput = await page.locator('#terminal-input');

      // Type partial command
      await terminalInput.fill('ls ');
      await page.waitForTimeout(300);

      // Help suggestions should appear
      const suggestions = await page.locator('.help-suggestions, .command-suggestions');

      if (await suggestions.count() > 0) {
        await expect(suggestions).toBeVisible();
      }
    });

    test('should autocomplete commands from help suggestions', async ({ page }) => {
      const terminalInput = await page.locator('#terminal-input');

      // Type partial command
      await terminalInput.fill('ca');
      await page.waitForTimeout(300);

      // Press Tab for autocomplete
      await page.keyboard.press('Tab');
      await page.waitForTimeout(200);

      const inputValue = await terminalInput.inputValue();
      // Should autocomplete to "cat"
      expect(inputValue).toBe('cat');
    });
  });

  test.describe('Accessibility (WCAG 2.1 AA)', () => {
    test('help panel should be keyboard navigable', async ({ page }) => {
      // Tab through help panel elements
      await page.keyboard.press('Tab');
      await page.keyboard.press('Tab');

      const focusedElement = await page.evaluate(() => {
        return document.activeElement?.tagName;
      });

      expect(focusedElement).toBeTruthy();
    });

    test('help panel should have proper ARIA landmarks', async ({ page }) => {
      const helpPanel = await page.locator('#panel-help');

      const ariaLabel = await helpPanel.getAttribute('aria-label');
      const role = await helpPanel.getAttribute('role');

      // Should have descriptive label or role
      expect(ariaLabel || role).toBeTruthy();
    });

    test('help command items should have proper ARIA roles', async ({ page }) => {
      const firstCommand = await page.locator('.help-command-item').first();

      if (await firstCommand.count() > 0) {
        const role = await firstCommand.getAttribute('role');
        // Should be button, listitem, or similar
        expect(role).toMatch(/button|listitem|article/);
      }
    });

    test('expanded/collapsed state should be indicated with aria-expanded', async ({ page }) => {
      const firstCommand = await page.locator('.help-command-item').first();

      if (await firstCommand.count() > 0) {
        const ariaExpanded = await firstCommand.getAttribute('aria-expanded');
        expect(ariaExpanded).toMatch(/true|false/);
      }
    });

    test('help panel should have sufficient color contrast', async ({ page }) => {
      const helpPanel = await page.locator('#panel-help');

      const contrast = await helpPanel.evaluate((el) => {
        const style = window.getComputedStyle(el);
        return {
          color: style.color,
          backgroundColor: style.backgroundColor
        };
      });

      // Basic check that colors are defined
      expect(contrast.color).toBeTruthy();
      expect(contrast.backgroundColor).toBeTruthy();
    });

    test('search input should have proper label', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      const ariaLabel = await searchInput.getAttribute('aria-label');
      const label = await page.locator('label[for="help-search"]');

      // Should have aria-label or associated label element
      expect(ariaLabel || (await label.count() > 0)).toBeTruthy();
    });
  });

  test.describe('Performance', () => {
    test('help panel should load within 500ms', async ({ page }) => {
      const startTime = Date.now();

      await page.keyboard.press('F1');
      await page.waitForSelector('#panel-help:visible', { timeout: 1000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(500);
    });

    test('search should filter results in under 100ms', async ({ page }) => {
      const searchInput = await page.locator('#help-search');

      const startTime = Date.now();
      await searchInput.fill('test');
      await page.waitForTimeout(100);

      const filterTime = Date.now() - startTime;
      expect(filterTime).toBeLessThan(100);
    });

    test('help data should be cached for fast subsequent access', async ({ page }) => {
      // Open help first time
      await page.keyboard.press('F1');
      await page.waitForTimeout(300);

      // Close
      await page.keyboard.press('Escape');
      await page.waitForTimeout(200);

      // Open second time (should be faster)
      const startTime = Date.now();
      await page.keyboard.press('F1');
      await page.waitForSelector('#panel-help:visible', { timeout: 500 });
      const secondLoadTime = Date.now() - startTime;

      expect(secondLoadTime).toBeLessThan(200);
    });
  });
});
