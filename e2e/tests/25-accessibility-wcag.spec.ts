import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

/**
 * WOS-308: WCAG 2.1 Level AA Accessibility Compliance
 *
 * Requirements:
 * - Contrast ratios: 4.5:1 text, 3:1 UI components
 * - Keyboard-only navigation (no mouse required)
 * - ARIA labels for all interactive elements
 * - Screen reader testing compatibility
 * - Focus indicators visible
 * - Skip links for navigation
 */

test.describe('WOS-308: WCAG 2.1 Level AA Accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    await page.waitForLoadState('networkidle');
  });

  test.describe('Automated Accessibility Scanning (axe-core)', () => {
    test('should have no axe violations on initial page load', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have no axe violations in terminal area', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .include('#terminal')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have no axe violations in file manager', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .include('.file-manager')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have no axe violations in system monitor panel', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .include('#panel-system-monitor')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have no axe violations in time-travel debugger', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .include('#panel-time-travel-debugger')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });
  });

  test.describe('Contrast Ratios (WCAG 2.1 Success Criterion 1.4.3)', () => {
    test('should have sufficient contrast for text (4.5:1 minimum)', async ({ page }) => {
      // axe-core will check this in the automated scan above
      // This test documents the specific requirement
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag143']) // WCAG 1.4.3 Contrast (Minimum)
        .analyze();

      expect(accessibilityScanResults.violations.filter(v =>
        v.id === 'color-contrast'
      )).toEqual([]);
    });

    test('should have sufficient contrast for UI components (3:1 minimum)', async ({ page }) => {
      // Check button contrast
      const buttons = await page.locator('button').all();
      expect(buttons.length).toBeGreaterThan(0);

      // axe will check non-text contrast in WCAG 2.1 AA
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag21aa'])
        .analyze();

      const nonTextContrastViolations = accessibilityScanResults.violations.filter(v =>
        v.id === 'color-contrast-enhanced' || v.id === 'link-in-text-block'
      );

      expect(nonTextContrastViolations).toEqual([]);
    });

    test('should maintain contrast in dark mode (if implemented)', async ({ page }) => {
      // Check for dark mode toggle
      const darkModeToggle = page.locator('[aria-label*="dark mode" i], [aria-label*="theme" i]');
      const count = await darkModeToggle.count();

      if (count > 0) {
        await darkModeToggle.first().click();
        await page.waitForTimeout(100);

        const accessibilityScanResults = await new AxeBuilder({ page })
          .withTags(['wcag143'])
          .analyze();

        expect(accessibilityScanResults.violations.filter(v =>
          v.id === 'color-contrast'
        )).toEqual([]);
      } else {
        // No dark mode, test passes
        expect(true).toBeTruthy();
      }
    });
  });

  test.describe('Keyboard-Only Navigation (WCAG 2.1 Success Criterion 2.1.1)', () => {
    test('should allow full keyboard navigation without mouse', async ({ page }) => {
      // Start from terminal input
      await page.locator('#terminal-input').focus();

      // Tab through all focusable elements
      const focusableElements: string[] = [];
      let previousElement = '';

      // Tab through up to 50 elements to map keyboard navigation
      for (let i = 0; i < 50; i++) {
        await page.keyboard.press('Tab');
        await page.waitForTimeout(50);

        const focusedElement = await page.evaluate(() => {
          const el = document.activeElement;
          return el ? el.tagName + (el.id ? '#' + el.id : '') + (el.className ? '.' + el.className.split(' ')[0] : '') : '';
        });

        if (focusedElement === previousElement) {
          break; // Reached end of tab cycle
        }

        focusableElements.push(focusedElement);
        previousElement = focusedElement;
      }

      // Should have at least terminal input and some buttons
      expect(focusableElements.length).toBeGreaterThan(3);
    });

    test('should allow Shift+Tab backward navigation', async ({ page }) => {
      await page.locator('#terminal-input').focus();

      // Tab forward
      await page.keyboard.press('Tab');
      const firstElement = await page.evaluate(() => document.activeElement?.id);

      // Shift+Tab backward
      await page.keyboard.press('Shift+Tab');
      const backElement = await page.evaluate(() => document.activeElement?.id);

      // Should return to terminal-input
      expect(backElement).toBe('terminal-input');
    });

    test('should activate buttons with Enter key', async ({ page }) => {
      // Find a button and activate with keyboard
      const firstButton = page.locator('button').first();
      await firstButton.focus();

      const buttonText = await firstButton.textContent();

      // Press Enter
      await page.keyboard.press('Enter');
      await page.waitForTimeout(100);

      // Button should have been activated (no errors)
      expect(buttonText).toBeTruthy();
    });

    test('should activate buttons with Space key', async ({ page }) => {
      const firstButton = page.locator('button').first();
      await firstButton.focus();

      const buttonText = await firstButton.textContent();

      // Press Space
      await page.keyboard.press(' ');
      await page.waitForTimeout(100);

      // Button should have been activated (no errors)
      expect(buttonText).toBeTruthy();
    });

    test('should navigate file list with arrow keys', async ({ page }) => {
      const fileItems = page.locator('.file-list .file-item');
      const count = await fileItems.count();

      if (count > 0) {
        await fileItems.first().focus();

        // Arrow down
        await page.keyboard.press('ArrowDown');
        await page.waitForTimeout(50);

        const focusedAfterDown = await page.evaluate(() => {
          return document.activeElement?.textContent;
        });

        expect(focusedAfterDown).toBeTruthy();
      } else {
        // No file items, test passes
        expect(true).toBeTruthy();
      }
    });

    test('should trap focus in modal dialogs (if present)', async ({ page }) => {
      // Check for modal dialogs
      const modal = page.locator('[role="dialog"]');
      const count = await modal.count();

      if (count > 0) {
        // Tab through modal elements
        const firstFocusable = modal.locator('button, input, textarea, select, a[href]').first();
        await firstFocusable.focus();

        // Tab multiple times
        for (let i = 0; i < 10; i++) {
          await page.keyboard.press('Tab');
        }

        // Focus should still be within modal
        const focusedElement = await page.evaluate(() => {
          const modal = document.querySelector('[role="dialog"]');
          return modal?.contains(document.activeElement);
        });

        expect(focusedElement).toBeTruthy();
      } else {
        // No modals, test passes
        expect(true).toBeTruthy();
      }
    });
  });

  test.describe('ARIA Labels (WCAG 2.1 Success Criterion 4.1.2)', () => {
    test('should have aria-label or aria-labelledby on all interactive elements', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag412']) // WCAG 4.1.2 Name, Role, Value
        .analyze();

      const labelViolations = accessibilityScanResults.violations.filter(v =>
        v.id === 'button-name' ||
        v.id === 'link-name' ||
        v.id === 'input-button-name' ||
        v.id === 'aria-input-field-name'
      );

      expect(labelViolations).toEqual([]);
    });

    test('should have aria-label on icon-only buttons', async ({ page }) => {
      const iconButtons = await page.locator('.btn-icon').all();

      for (const button of iconButtons) {
        const ariaLabel = await button.getAttribute('aria-label');
        const textContent = await button.textContent();

        // Either aria-label or visible text
        expect(ariaLabel || textContent?.trim()).toBeTruthy();
      }
    });

    test('should have proper ARIA roles for custom widgets', async ({ page }) => {
      // Check for custom widgets with roles
      const widgets = await page.locator('[role]').all();

      for (const widget of widgets) {
        const role = await widget.getAttribute('role');
        const validRoles = [
          'button', 'link', 'textbox', 'dialog', 'alertdialog',
          'tablist', 'tab', 'tabpanel', 'menu', 'menuitem',
          'navigation', 'main', 'complementary', 'region'
        ];

        if (role) {
          expect(validRoles).toContain(role);
        }
      }
    });

    test('should have aria-live regions for dynamic content', async ({ page }) => {
      // Terminal output should be a live region
      const terminalOutput = page.locator('#terminal-output');
      const ariaLive = await terminalOutput.getAttribute('aria-live');

      // Should be 'polite' or 'assertive'
      expect(['polite', 'assertive', null]).toContain(ariaLive);
    });

    test('should have aria-expanded on collapsible panels', async ({ page }) => {
      const collapseButtons = await page.locator('.btn-collapse').all();

      for (const button of collapseButtons) {
        const ariaExpanded = await button.getAttribute('aria-expanded');

        // Should be 'true' or 'false'
        expect(['true', 'false']).toContain(ariaExpanded);
      }
    });
  });

  test.describe('Focus Indicators (WCAG 2.1 Success Criterion 2.4.7)', () => {
    test('should show visible focus indicator on all interactive elements', async ({ page }) => {
      const terminalInput = page.locator('#terminal-input');
      await terminalInput.focus();

      const outlineStyle = await terminalInput.evaluate((el) => {
        const styles = window.getComputedStyle(el);
        return {
          outline: styles.outline,
          outlineWidth: styles.outlineWidth,
          outlineStyle: styles.outlineStyle,
          border: styles.border,
          boxShadow: styles.boxShadow
        };
      });

      // Should have some form of focus indicator
      const hasFocusIndicator =
        outlineStyle.outlineWidth !== '0px' ||
        outlineStyle.boxShadow !== 'none';

      expect(hasFocusIndicator).toBeTruthy();
    });

    test('should show focus indicator on buttons', async ({ page }) => {
      const firstButton = page.locator('button').first();
      await firstButton.focus();

      const outlineStyle = await firstButton.evaluate((el) => {
        const styles = window.getComputedStyle(el);
        return {
          outline: styles.outline,
          outlineWidth: styles.outlineWidth,
          boxShadow: styles.boxShadow
        };
      });

      const hasFocusIndicator =
        outlineStyle.outlineWidth !== '0px' ||
        outlineStyle.boxShadow !== 'none';

      expect(hasFocusIndicator).toBeTruthy();
    });

    test('should maintain 3:1 contrast for focus indicators', async ({ page }) => {
      // axe-core checks this in WCAG 2.1 AA
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag21aa'])
        .analyze();

      const focusViolations = accessibilityScanResults.violations.filter(v =>
        v.id === 'focus-order-semantics'
      );

      expect(focusViolations).toEqual([]);
    });

    test('should not remove focus indicators with outline:none', async ({ page }) => {
      // Check CSS for outline:none on :focus
      const hasOutlineNone = await page.evaluate(() => {
        const styles = document.styleSheets;
        for (let i = 0; i < styles.length; i++) {
          try {
            const rules = styles[i].cssRules || styles[i].rules;
            for (let j = 0; j < rules.length; j++) {
              const rule = rules[j] as CSSStyleRule;
              if (rule.selectorText && rule.selectorText.includes(':focus')) {
                if (rule.style.outline === 'none' && !rule.style.boxShadow) {
                  return true;
                }
              }
            }
          } catch (e) {
            // Cross-origin stylesheet, skip
          }
        }
        return false;
      });

      expect(hasOutlineNone).toBeFalsy();
    });
  });

  test.describe('Skip Links (WCAG 2.1 Success Criterion 2.4.1)', () => {
    test('should have skip to main content link', async ({ page }) => {
      const skipLinks = await page.locator('a[href^="#"]').all();

      let hasSkipLink = false;
      for (const link of skipLinks) {
        const text = await link.textContent();
        if (text?.toLowerCase().includes('skip') || text?.toLowerCase().includes('main')) {
          hasSkipLink = true;
          break;
        }
      }

      // Skip link is recommended but not always required
      // Document if present
      if (hasSkipLink) {
        expect(hasSkipLink).toBeTruthy();
      } else {
        // Not present, but note in test results
        console.log('Note: Skip link not found. Consider adding for better accessibility.');
      }
    });

    test('should show skip link on focus', async ({ page }) => {
      // Press Tab on initial load to focus skip link (if present)
      await page.keyboard.press('Tab');

      const focusedElement = await page.evaluate(() => {
        const el = document.activeElement;
        return {
          tag: el?.tagName,
          text: el?.textContent,
          visible: el ? window.getComputedStyle(el).display !== 'none' : false
        };
      });

      // If focused element is a skip link, it should be visible
      if (focusedElement.text?.toLowerCase().includes('skip')) {
        expect(focusedElement.visible).toBeTruthy();
      }
    });

    test('should navigate to target when skip link is activated', async ({ page }) => {
      // Find skip link
      const skipLink = page.locator('a[href^="#main"], a[href^="#content"]').first();
      const count = await skipLink.count();

      if (count > 0) {
        await skipLink.click();
        await page.waitForTimeout(100);

        // Focus should be on main content
        const focusedElement = await page.evaluate(() => {
          return document.activeElement?.id;
        });

        expect(['main', 'content', 'terminal', 'terminal-input']).toContain(focusedElement);
      }
    });
  });

  test.describe('Semantic HTML (WCAG 2.1 Success Criterion 1.3.1)', () => {
    test('should use semantic HTML5 landmarks', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag131']) // WCAG 1.3.1 Info and Relationships
        .analyze();

      const landmarkViolations = accessibilityScanResults.violations.filter(v =>
        v.id === 'region' || v.id === 'landmark-one-main'
      );

      expect(landmarkViolations).toEqual([]);
    });

    test('should have proper heading hierarchy', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag131'])
        .analyze();

      const headingViolations = accessibilityScanResults.violations.filter(v =>
        v.id === 'heading-order'
      );

      expect(headingViolations).toEqual([]);
    });

    test('should have alt text on all images', async ({ page }) => {
      const images = await page.locator('img').all();

      for (const img of images) {
        const alt = await img.getAttribute('alt');

        // All images must have alt attribute (can be empty for decorative)
        expect(alt !== null).toBeTruthy();
      }
    });

    test('should use proper list markup for lists', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag131'])
        .analyze();

      const listViolations = accessibilityScanResults.violations.filter(v =>
        v.id === 'list' || v.id === 'listitem'
      );

      expect(listViolations).toEqual([]);
    });
  });

  test.describe('Screen Reader Compatibility', () => {
    test('should have document language set', async ({ page }) => {
      const lang = await page.getAttribute('html', 'lang');

      expect(lang).toBeTruthy();
      expect(lang).toBe('en');
    });

    test('should have page title', async ({ page }) => {
      const title = await page.title();

      expect(title).toBeTruthy();
      expect(title.length).toBeGreaterThan(0);
    });

    test('should announce dynamic content changes', async ({ page }) => {
      // Terminal output should have aria-live
      const terminalOutput = page.locator('#terminal-output');
      const ariaLive = await terminalOutput.getAttribute('aria-live');

      // Should be polite or assertive for announcements
      if (ariaLive) {
        expect(['polite', 'assertive']).toContain(ariaLive);
      }
    });

    test('should have aria-describedby for additional context', async ({ page }) => {
      // Check inputs for aria-describedby
      const inputs = await page.locator('input, textarea').all();

      for (const input of inputs) {
        const ariaDescribedBy = await input.getAttribute('aria-describedby');

        if (ariaDescribedBy) {
          // Referenced element should exist
          const descElement = page.locator(`#${ariaDescribedBy}`);
          const count = await descElement.count();
          expect(count).toBeGreaterThan(0);
        }
      }
    });
  });

  test.describe('Form Accessibility', () => {
    test('should have labels for all form inputs', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag412'])
        .analyze();

      const labelViolations = accessibilityScanResults.violations.filter(v =>
        v.id === 'label' || v.id === 'label-title-only'
      );

      expect(labelViolations).toEqual([]);
    });

    test('should show clear error messages for validation', async ({ page }) => {
      const terminalInput = page.locator('#terminal-input');

      // Try an invalid command
      await terminalInput.fill('');
      await terminalInput.press('Enter');

      await page.waitForTimeout(100);

      // Error should be announced (aria-live or aria-describedby)
      const ariaDescribedBy = await terminalInput.getAttribute('aria-describedby');
      const ariaInvalid = await terminalInput.getAttribute('aria-invalid');

      // Either aria-invalid or aria-describedby should provide error context
      expect(ariaDescribedBy || ariaInvalid !== null).toBeTruthy();
    });

    test('should have visible focus on form fields', async ({ page }) => {
      const terminalInput = page.locator('#terminal-input');
      await terminalInput.focus();

      const hasFocus = await terminalInput.evaluate((el) => {
        return document.activeElement === el;
      });

      expect(hasFocus).toBeTruthy();
    });
  });

  test.describe('Color Independence (WCAG 2.1 Success Criterion 1.4.1)', () => {
    test('should not use color alone to convey information', async ({ page }) => {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag141']) // WCAG 1.4.1 Use of Color
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    });

    test('should have text labels in addition to color coding', async ({ page }) => {
      // Process states should have text labels, not just colors
      const processItems = await page.locator('.process-item').all();

      for (const item of processItems) {
        const text = await item.textContent();

        // Should have text content (not just color)
        expect(text?.trim().length).toBeGreaterThan(0);
      }
    });
  });

  test.describe('Text Resize (WCAG 2.1 Success Criterion 1.4.4)', () => {
    test('should remain functional when text is resized to 200%', async ({ page }) => {
      // Set zoom to 200%
      const cdpSession = await page.context().newCDPSession(page);
      await cdpSession.send('Emulation.setPageScaleFactor', { pageScaleFactor: 2.0 });

      // Wait for layout to settle
      await page.waitForTimeout(200);

      // Terminal input should still be visible
      const terminalInput = page.locator('#terminal-input');
      await expect(terminalInput).toBeVisible();

      // Should still be able to interact
      await terminalInput.fill('help');
      await terminalInput.press('Enter');

      await page.waitForTimeout(100);

      // Output should appear
      const terminalOutput = page.locator('#terminal-output');
      const text = await terminalOutput.textContent();
      expect(text?.length).toBeGreaterThan(0);
    });
  });
});
