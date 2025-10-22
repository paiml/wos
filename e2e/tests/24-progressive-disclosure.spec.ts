import { test, expect } from '@playwright/test';

test.describe('Progressive Disclosure UI Layout (WOS-306)', () => {
  test.beforeEach(async ({ page }) => {
    // Enable tracing for debugging
    await page.goto('index.html?trace=DEBUG&categories=INIT,WASM,CONFIG,PANEL');

    // Set up console listener to capture trace output
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('[INIT]') || text.includes('[WASM]') || text.includes('[CONFIG]') || text.includes('[PANEL]')) {
        console.log(text);
      }
    });

    // Clear localStorage before each test and mark tutorial as completed to prevent auto-start
    await page.evaluate(() => {
      localStorage.clear();
      localStorage.setItem('wos-tutorial-completed', 'true');
    });
    await page.reload();
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test.describe('Initial Layout Defaults', () => {
    test('should have Terminal container visible by default', async ({ page }) => {
      const terminalContainer = page.locator('.terminal-container');

      // Terminal should be visible
      await expect(terminalContainer).toBeVisible();

      // Terminal should NOT have collapsed class
      const hasCollapsedClass = await terminalContainer.evaluate((el) =>
        el.classList.contains('collapsed')
      );
      expect(hasCollapsedClass).toBe(false);
    });

    test('should have Learning Objectives panel expanded by default', async ({ page }) => {
      const learningPanel = page.locator('[data-panel="learning_objectives"]');
      const learningContent = learningPanel.locator('.panel-content');

      // Learning Objectives should be visible
      await expect(learningPanel).toBeVisible();
      await expect(learningContent).toBeVisible();

      // Learning Objectives should NOT have collapsed class
      const hasCollapsedClass = await learningPanel.evaluate((el) =>
        el.classList.contains('collapsed')
      );
      expect(hasCollapsedClass).toBe(false);
    });

    test('should have System Monitor panel collapsed by default', async ({ page }) => {
      const systemMonitorPanel = page.locator('[data-panel="system_monitor_detailed"]');
      const systemMonitorContent = systemMonitorPanel.locator('.panel-content');

      // Panel element should exist in DOM
      await expect(systemMonitorPanel).toBeVisible();

      // Panel content should be hidden (collapsed)
      await expect(systemMonitorContent).not.toBeVisible();

      // Panel should have collapsed class
      const hasCollapsedClass = await systemMonitorPanel.evaluate((el) =>
        el.classList.contains('collapsed')
      );
      expect(hasCollapsedClass).toBe(true);
    });

    test('should have Time-Travel Debugger panel collapsed by default', async ({ page }) => {
      const timeTravelPanel = page.locator('[data-panel="time_travel_debugger"]');
      const timeTravelContent = timeTravelPanel.locator('.panel-content');

      // Panel element should exist in DOM
      await expect(timeTravelPanel).toBeVisible();

      // Panel content should be hidden (collapsed)
      await expect(timeTravelContent).not.toBeVisible();

      // Panel should have collapsed class
      const hasCollapsedClass = await timeTravelPanel.evaluate((el) =>
        el.classList.contains('collapsed')
      );
      expect(hasCollapsedClass).toBe(true);
    });

    test('should reduce initial cognitive load by hiding advanced panels', async ({ page }) => {
      // Count visible panel contents on initial load
      const visiblePanelContents = await page.locator('.panel-content:visible').count();

      // Only Learning Objectives should be visible initially
      // This reduces cognitive load by hiding advanced panels
      expect(visiblePanelContents).toBeLessThanOrEqual(2);

      // Verify Terminal container is visible
      const terminalVisible = await page.locator('.terminal-container').isVisible();
      const learningVisible = await page.locator('[data-panel="learning_objectives"] .panel-content').isVisible();

      expect(terminalVisible).toBe(true);
      expect(learningVisible).toBe(true);
    });
  });

  test.describe('Panel Collapse/Expand Behavior', () => {
    test('should expand System Monitor when collapse button clicked', async ({ page }) => {
      const systemMonitorPanel = page.locator('[data-panel="system_monitor_detailed"]');
      const collapseBtn = systemMonitorPanel.locator('.btn-collapse');
      const panelContent = systemMonitorPanel.locator('.panel-content');

      // Verify panel is collapsed initially
      await expect(panelContent).not.toBeVisible();

      // Click collapse button to expand
      await collapseBtn.click();
      await page.waitForTimeout(300); // Wait for animation

      // Verify panel content is now visible
      await expect(panelContent).toBeVisible();

      // Verify panel does not have collapsed class
      const hasCollapsedClass = await systemMonitorPanel.evaluate((el) =>
        el.classList.contains('collapsed')
      );
      expect(hasCollapsedClass).toBe(false);
    });

    test('should expand Time-Travel panel when collapse button clicked', async ({ page }) => {
      const timeTravelPanel = page.locator('[data-panel="time_travel_debugger"]');
      const collapseBtn = timeTravelPanel.locator('.btn-collapse');
      const panelContent = timeTravelPanel.locator('.panel-content');

      // Verify panel is collapsed initially
      await expect(panelContent).not.toBeVisible();

      // Click collapse button to expand
      await collapseBtn.click();
      await page.waitForTimeout(300); // Wait for animation

      // Verify panel content is now visible
      await expect(panelContent).toBeVisible();

      // Verify panel does not have collapsed class
      const hasCollapsedClass = await timeTravelPanel.evaluate((el) =>
        el.classList.contains('collapsed')
      );
      expect(hasCollapsedClass).toBe(false);
    });

    test('should toggle System Monitor panel when collapse button clicked twice', async ({ page }) => {
      const systemMonitorPanel = page.locator('[data-panel="system_monitor_detailed"]');
      const collapseBtn = systemMonitorPanel.locator('.btn-collapse');
      const panelContent = systemMonitorPanel.locator('.panel-content');

      // First expand the panel (it's collapsed by default)
      await collapseBtn.click();
      await page.waitForTimeout(300);

      // Verify panel is expanded
      await expect(panelContent).toBeVisible();

      // Click collapse button again to collapse
      await collapseBtn.click();
      await page.waitForTimeout(300); // Wait for animation

      // Verify panel content is hidden
      await expect(panelContent).not.toBeVisible();

      // Verify panel has collapsed class
      const hasCollapsedClass = await systemMonitorPanel.evaluate((el) =>
        el.classList.contains('collapsed')
      );
      expect(hasCollapsedClass).toBe(true);
    });

    test('should collapse Learning Objectives panel when collapse button clicked', async ({ page }) => {
      const learningPanel = page.locator('[data-panel="learning_objectives"]');
      const collapseBtn = learningPanel.locator('.btn-collapse');
      const panelContent = learningPanel.locator('.panel-content');

      // Verify panel is expanded initially
      await expect(panelContent).toBeVisible();

      // Click collapse button to collapse
      await collapseBtn.click();
      await page.waitForTimeout(300); // Wait for animation

      // Verify panel content is hidden
      await expect(panelContent).not.toBeVisible();

      // Verify panel has collapsed class
      const hasCollapsedClass = await learningPanel.evaluate((el) =>
        el.classList.contains('collapsed')
      );
      expect(hasCollapsedClass).toBe(true);
    });
  });

  test.describe('Tab-Based Navigation', () => {
    test('should have tab navigation for System Monitor panel when collapsed', async ({ page }) => {
      const systemMonitorPanel = page.locator('[data-panel="system_monitor_detailed"]');

      // Panel should be collapsed initially
      const panelContent = systemMonitorPanel.locator('.panel-content');
      await expect(panelContent).not.toBeVisible();

      // Should have tab element for quick access
      const panelTab = systemMonitorPanel.locator('.panel-tab');
      await expect(panelTab).toBeVisible();

      // Tab should be clickable
      await panelTab.click();
      await page.waitForTimeout(300); // Wait for animation

      // Panel should expand when tab clicked
      await expect(panelContent).toBeVisible();
    });

    test('should have tab navigation for Time-Travel panel when collapsed', async ({ page }) => {
      const timeTravelPanel = page.locator('[data-panel="time_travel_debugger"]');

      // Panel should be collapsed initially
      const panelContent = timeTravelPanel.locator('.panel-content');
      await expect(panelContent).not.toBeVisible();

      // Should have tab element for quick access
      const panelTab = timeTravelPanel.locator('.panel-tab');
      await expect(panelTab).toBeVisible();

      // Tab should be clickable
      await panelTab.click();
      await page.waitForTimeout(300); // Wait for animation

      // Panel should expand when tab clicked
      await expect(panelContent).toBeVisible();
    });

    test('should show tab labels with panel names', async ({ page }) => {
      // System Monitor tab
      const systemMonitorTab = page.locator('[data-panel="system_monitor_detailed"] .panel-tab');
      const systemMonitorTabText = await systemMonitorTab.textContent();
      expect(systemMonitorTabText).toContain('System Monitor');

      // Time-Travel tab
      const timeTravelTab = page.locator('[data-panel="time_travel_debugger"] .panel-tab');
      const timeTravelTabText = await timeTravelTab.textContent();
      expect(timeTravelTabText).toContain('Time-Travel');
    });

    test('should hide tabs when panels are expanded', async ({ page }) => {
      const systemMonitorPanel = page.locator('[data-panel="system_monitor_detailed"]');
      const collapseBtn = systemMonitorPanel.locator('.btn-collapse');
      const panelTab = systemMonitorPanel.locator('.panel-tab');

      // Panel is collapsed initially, tab should be visible
      await expect(panelTab).toBeVisible();

      // Expand panel
      await collapseBtn.click();
      await page.waitForTimeout(300);

      // Tab should be hidden when panel is expanded
      await expect(panelTab).not.toBeVisible();
    });

    test('should support keyboard navigation on tabs', async ({ page }) => {
      const systemMonitorTab = page.locator('[data-panel="system_monitor_detailed"] .panel-tab');

      // Tab should be focusable
      await systemMonitorTab.focus();

      // Check ARIA attributes for accessibility
      const hasTabRole = await systemMonitorTab.evaluate((el) =>
        el.getAttribute('role') === 'tab'
      );
      expect(hasTabRole).toBe(true);

      // Tab should have aria-expanded attribute
      const ariaExpanded = await systemMonitorTab.getAttribute('aria-expanded');
      expect(ariaExpanded).toBe('false');

      // Enter key should expand panel
      await systemMonitorTab.press('Enter');
      await page.waitForTimeout(300);

      // Panel content should be visible
      const panelContent = page.locator('[data-panel="system_monitor_detailed"] .panel-content');
      await expect(panelContent).toBeVisible();
    });
  });

  test.describe('Layout Preference Persistence', () => {
    test('should persist panel collapse state in localStorage', async ({ page }) => {
      // Collapse Learning Objectives panel
      const learningPanel = page.locator('[data-panel="learning_objectives"]');
      const collapseBtn = learningPanel.locator('.btn-collapse');
      await collapseBtn.click();
      await page.waitForTimeout(300);

      // Check localStorage
      const layoutPrefs = await page.evaluate(() => {
        return localStorage.getItem('wos-layout-preferences');
      });

      expect(layoutPrefs).not.toBeNull();

      const prefs = JSON.parse(layoutPrefs);
      expect(prefs.learning_objectives).toBe('collapsed');
    });

    test('should restore panel states from localStorage on page load', async ({ page }) => {
      // Set initial preferences
      await page.evaluate(() => {
        localStorage.setItem('wos-layout-preferences', JSON.stringify({
          learning_objectives: 'collapsed',
          system_monitor_detailed: 'expanded',
          time_travel_debugger: 'expanded'
        }));
      });

      // Reload page
      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
      await page.waitForTimeout(300); // Wait for layout restoration

      // Verify states match preferences
      const learningContent = page.locator('[data-panel="learning_objectives"] .panel-content');
      const visualContent = page.locator('[data-panel="system_monitor_detailed"] .panel-content');
      const timeTravelContent = page.locator('[data-panel="time_travel_debugger"] .panel-content');

      await expect(learningContent).not.toBeVisible();
      await expect(visualContent).toBeVisible();
      await expect(timeTravelContent).toBeVisible();
    });

    test('should update localStorage when expanding collapsed panel', async ({ page }) => {
      // System Monitor is collapsed by default
      const systemMonitorPanel = page.locator('[data-panel="system_monitor_detailed"]');
      const collapseBtn = systemMonitorPanel.locator('.btn-collapse');

      // Expand panel
      await collapseBtn.click();
      await page.waitForTimeout(300);

      // Check localStorage updated
      const layoutPrefs = await page.evaluate(() => {
        return localStorage.getItem('wos-layout-preferences');
      });

      const prefs = JSON.parse(layoutPrefs);
      expect(prefs.system_monitor_detailed).toBe('expanded');
    });

    test('should persist preferences across multiple panel changes', async ({ page }) => {
      // Make multiple changes
      await page.locator('[data-panel="learning_objectives"] .btn-collapse').click();
      await page.waitForTimeout(300);

      await page.locator('[data-panel="system_monitor_detailed"] .btn-collapse').click();
      await page.waitForTimeout(300);

      await page.locator('[data-panel="time_travel_debugger"] .btn-collapse').click();
      await page.waitForTimeout(300);

      // Reload page
      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
      await page.waitForTimeout(300);

      // Verify all changes persisted
      const learningContent = page.locator('[data-panel="learning_objectives"] .panel-content');
      const visualContent = page.locator('[data-panel="system_monitor_detailed"] .panel-content');
      const timeTravelContent = page.locator('[data-panel="time_travel_debugger"] .panel-content');

      await expect(learningContent).not.toBeVisible(); // Collapsed
      await expect(visualContent).toBeVisible(); // Expanded
      await expect(timeTravelContent).toBeVisible(); // Expanded
    });
  });

  test.describe('Reduced Motion Support', () => {
    test('should disable animations when prefers-reduced-motion is set', async ({ page }) => {
      // Emulate prefers-reduced-motion
      await page.emulateMedia({ reducedMotion: 'reduce' });

      const systemMonitorPanel = page.locator('[data-panel="system_monitor_detailed"]');
      const collapseBtn = systemMonitorPanel.locator('.btn-collapse');
      const panelContent = systemMonitorPanel.locator('.panel-content');

      // Expand panel
      await collapseBtn.click();

      // Without animations, panel should be visible immediately (no timeout needed)
      // Check that no transition is applied
      const transitionDuration = await panelContent.evaluate((el) => {
        return window.getComputedStyle(el).transitionDuration;
      });

      // Duration should be 0s or very small (browser default)
      expect(transitionDuration === '0s' || transitionDuration === '0ms').toBe(true);
    });

    test('should enable smooth animations when prefers-reduced-motion is not set', async ({ page }) => {
      // Emulate no reduced motion preference (default)
      await page.emulateMedia({ reducedMotion: 'no-preference' });

      const systemMonitorPanel = page.locator('[data-panel="system_monitor_detailed"]');
      const collapseBtn = systemMonitorPanel.locator('.btn-collapse');
      const panelContent = systemMonitorPanel.locator('.panel-content');

      // Check transition is applied
      const transitionDuration = await panelContent.evaluate((el) => {
        return window.getComputedStyle(el).transitionDuration;
      });

      // Duration should be greater than 0
      expect(transitionDuration !== '0s' && transitionDuration !== '0ms').toBe(true);
    });

    test('should respect system reduced motion preference for tab navigation', async ({ page }) => {
      // Emulate prefers-reduced-motion
      await page.emulateMedia({ reducedMotion: 'reduce' });

      const systemMonitorTab = page.locator('[data-panel="system_monitor_detailed"] .panel-tab');

      // Click tab to expand
      await systemMonitorTab.click();

      // Check that tab expand animation is disabled
      const panelContent = page.locator('[data-panel="system_monitor_detailed"] .panel-content');
      const transitionDuration = await panelContent.evaluate((el) => {
        return window.getComputedStyle(el).transitionDuration;
      });

      expect(transitionDuration === '0s' || transitionDuration === '0ms').toBe(true);
    });
  });

  test.describe('E2E Progressive Disclosure', () => {
    test('should guide user through progressive interface discovery', async ({ page }) => {
      // 1. User starts with simple interface (Terminal + Learning Objectives)
      const terminalContainer = page.locator('.terminal-container');
      const learningContent = page.locator('[data-panel="learning_objectives"] .panel-content');

      await expect(terminalContainer).toBeVisible();
      await expect(learningContent).toBeVisible();

      // 2. User discovers System Monitor tab
      const systemMonitorTab = page.locator('[data-panel="system_monitor_detailed"] .panel-tab');
      await expect(systemMonitorTab).toBeVisible();

      // 3. User clicks tab to reveal more functionality
      await systemMonitorTab.click();
      await page.waitForTimeout(300);

      const visualContent = page.locator('[data-panel="system_monitor_detailed"] .panel-content');
      await expect(visualContent).toBeVisible();

      // 4. User discovers Time-Travel tab
      const timeTravelTab = page.locator('[data-panel="time_travel_debugger"] .panel-tab');
      await expect(timeTravelTab).toBeVisible();

      // 5. User expands Time-Travel for advanced debugging
      await timeTravelTab.click();
      await page.waitForTimeout(300);

      const timeTravelContent = page.locator('[data-panel="time_travel_debugger"] .panel-content');
      await expect(timeTravelContent).toBeVisible();

      // 6. Preferences are saved
      const layoutPrefs = await page.evaluate(() => {
        return localStorage.getItem('wos-layout-preferences');
      });
      expect(layoutPrefs).not.toBeNull();

      const prefs = JSON.parse(layoutPrefs);
      expect(prefs.system_monitor_detailed).toBe('expanded');
      expect(prefs.time_travel_debugger).toBe('expanded');
    });

    test('should support power users who want all panels visible', async ({ page }) => {
      // Expand all initially collapsed panels
      await page.locator('[data-panel="system_monitor_detailed"] .btn-collapse').click();
      await page.waitForTimeout(400);

      await page.locator('[data-panel="time_travel_debugger"] .btn-collapse').click();
      await page.waitForTimeout(400);

      // Multiple panels should now be visible (learning_objectives + expanded panels)
      // After expanding these 2 panels, we should have at least 3 visible
      const allPanelContents = await page.locator('.panel-content:visible').count();
      expect(allPanelContents).toBeGreaterThanOrEqual(3);

      // Preferences should persist
      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
      await page.waitForTimeout(400);

      const allPanelContentsAfterReload = await page.locator('.panel-content:visible').count();
      expect(allPanelContentsAfterReload).toBeGreaterThanOrEqual(3);
    });

    test('should help beginners stay focused by hiding complexity', async ({ page }) => {
      // On fresh load, check that advanced panels are hidden
      const visualContent = page.locator('[data-panel="system_monitor_detailed"] .panel-content');
      const timeTravelContent = page.locator('[data-panel="time_travel_debugger"] .panel-content');

      await expect(visualContent).not.toBeVisible();
      await expect(timeTravelContent).not.toBeVisible();

      // Beginner only sees essential interface (Learning Objectives should be visible)
      const learningVisible = await page.locator('[data-panel="learning_objectives"] .panel-content').isVisible();
      expect(learningVisible).toBe(true);
    });
  });
});
