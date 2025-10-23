// Visual UX Testing with Playwright
// Takes screenshots to evaluate actual UI appearance

const { test, expect } = require('@playwright/test');

test.describe('WOS Panel UX Visual Testing', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the local dev server
    await page.goto('http://127.0.0.1:8000');

    // Clear localStorage to ensure clean state (no stale panel positions)
    await page.evaluate(() => {
      localStorage.clear();
    });

    // Reload to apply clean state
    await page.reload();

    // Wait for WASM to load
    await page.waitForTimeout(2000);

    // Dismiss tutorial modal if present
    const skipButton = page.locator('button:has-text("Skip Tutorial")');
    if (await skipButton.isVisible()) {
      await skipButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('screenshot initial state - all panels', async ({ page }) => {
    // Take full page screenshot
    await page.screenshot({
      path: 'tests/e2e/screenshots/01-initial-state.png',
      fullPage: true
    });

    console.log('Screenshot saved: 01-initial-state.png');
  });

  test('screenshot collapsed panels', async ({ page }) => {
    // Find all collapse buttons and click them to collapse all panels
    const collapseButtons = await page.locator('.btn-collapse').all();

    for (const btn of collapseButtons) {
      await btn.click({ force: true }); // Force click to bypass help button overlay
      await page.waitForTimeout(300); // Wait for animation
    }

    await page.screenshot({
      path: 'tests/e2e/screenshots/02-all-collapsed.png',
      fullPage: true
    });

    console.log('Screenshot saved: 02-all-collapsed.png');
  });

  test('screenshot expand first panel', async ({ page }) => {
    // Collapse all first
    const collapseButtons = await page.locator('.btn-collapse').all();
    for (const btn of collapseButtons) {
      await btn.click({ force: true }); // Force click to bypass help button
      await page.waitForTimeout(100);
    }

    // Expand first panel
    const firstPanel = page.locator('[data-panel]').first();
    const firstCollapseBtn = firstPanel.locator('.btn-collapse');
    await firstCollapseBtn.click({ force: true });
    await page.waitForTimeout(300);

    await page.screenshot({
      path: 'tests/e2e/screenshots/03-first-panel-expanded.png',
      fullPage: true
    });

    console.log('Screenshot saved: 03-first-panel-expanded.png');
  });

  test('screenshot expand different panels sequentially', async ({ page }) => {
    const panels = await page.locator('[data-panel]').all();

    let index = 1;
    for (const panel of panels.slice(0, 3)) {  // Test first 3 panels
      const panelName = await panel.getAttribute('data-panel');
      const collapseBtn = panel.locator('.btn-collapse');

      await collapseBtn.click();
      await page.waitForTimeout(300);

      await page.screenshot({
        path: `tests/e2e/screenshots/04-panel-${index}-${panelName}.png`,
        fullPage: true
      });

      console.log(`Screenshot saved: 04-panel-${index}-${panelName}.png`);
      index++;
    }
  });

  test('measure panel heights and spacing', async ({ page }) => {
    const panels = await page.locator('[data-panel]').all();

    const measurements = [];
    for (const panel of panels) {
      const box = await panel.boundingBox();
      const panelName = await panel.getAttribute('data-panel');
      const isCollapsed = await panel.evaluate(el => el.classList.contains('collapsed'));

      measurements.push({
        panel: panelName,
        collapsed: isCollapsed,
        height: box ? box.height : 0,
        y: box ? box.y : 0
      });
    }

    console.log('Panel measurements:', JSON.stringify(measurements, null, 2));

    // Check if panels are off-screen
    const viewport = page.viewportSize();
    const offscreen = measurements.filter(m => m.y + m.height > viewport.height);

    if (offscreen.length > 0) {
      console.log('PROBLEM: Panels extending off-screen:', offscreen);
    }
  });

  test('test accordion behavior visually', async ({ page }) => {
    // Test that expanding one panel collapses others
    const processListPanel = page.locator('[data-panel="process_list"]');
    const memoryMapPanel = page.locator('[data-panel="memory_map"]');

    // Expand process list
    await processListPanel.locator('.btn-collapse').click();
    await page.waitForTimeout(300);

    await page.screenshot({
      path: 'tests/e2e/screenshots/05-accordion-process-list.png',
      fullPage: true
    });

    // Now expand memory map - should collapse process list
    await memoryMapPanel.locator('.btn-collapse').click();
    await page.waitForTimeout(300);

    await page.screenshot({
      path: 'tests/e2e/screenshots/06-accordion-memory-map.png',
      fullPage: true
    });

    // Check if process list is actually collapsed
    const processListCollapsed = await processListPanel.evaluate(el =>
      el.classList.contains('collapsed')
    );

    console.log('Process list collapsed after expanding memory map:', processListCollapsed);
    expect(processListCollapsed).toBe(true);
  });

  test('verify accordion exclusivity - only one panel expanded', async ({ page }) => {
    console.log('\n=== ACCORDION EXCLUSIVITY TEST ===\n');

    // Count how many panels are expanded initially
    const panels = await page.locator('[data-panel]').all();
    let expandedCount = 0;
    let expandedNames = [];

    for (const panel of panels) {
      const name = await panel.getAttribute('data-panel');
      const isCollapsed = await panel.evaluate(el => el.classList.contains('collapsed'));
      if (!isCollapsed) {
        expandedCount++;
        expandedNames.push(name);
      }
    }

    console.log(`Initially expanded panels: ${expandedCount}`);
    console.log(`Expanded panel names: ${expandedNames.join(', ')}`);
    expect(expandedCount).toBe(1); // MUST be exactly 1

    // Test expanding different panels
    const testPanels = ['process_list', 'memory_map', 'syscall_trace'];
    for (const panelName of testPanels) {
      const panel = page.locator(`[data-panel="${panelName}"]`);
      await panel.locator('.btn-collapse').click({ force: true });
      await page.waitForTimeout(300);

      // Count expanded panels again
      let currentExpandedCount = 0;
      let currentExpanded = [];
      for (const p of panels) {
        const name = await p.getAttribute('data-panel');
        const isCollapsed = await p.evaluate(el => el.classList.contains('collapsed'));
        if (!isCollapsed) {
          currentExpandedCount++;
          currentExpanded.push(name);
        }
      }

      console.log(`After expanding ${panelName}: ${currentExpandedCount} expanded (${currentExpanded.join(', ')})`);
      expect(currentExpandedCount).toBe(1); // MUST ALWAYS be exactly 1
    }

    console.log('✅ Accordion exclusivity verified - always exactly 1 panel expanded');
  });

  test('analyze content visibility and scrollability', async ({ page }) => {
    console.log('\n=== CONTENT VISIBILITY ANALYSIS ===\n');

    // Expand Process List and check how many rows are visible
    const processListPanel = page.locator('[data-panel="process_list"]');
    await processListPanel.locator('.btn-collapse').click();
    await page.waitForTimeout(300);

    const processRows = await processListPanel.locator('.process-table tbody tr').count();
    const panelHeight = await processListPanel.evaluate(el => el.getBoundingClientRect().height);
    const contentHeight = await processListPanel.locator('.panel-content').evaluate(el => {
      return {
        scrollHeight: el.scrollHeight,
        clientHeight: el.clientHeight,
        scrollable: el.scrollHeight > el.clientHeight
      };
    });

    console.log(`Process List Panel:`);
    console.log(`  Panel height: ${panelHeight}px`);
    console.log(`  Content scroll height: ${contentHeight.scrollHeight}px`);
    console.log(`  Content client height: ${contentHeight.clientHeight}px`);
    console.log(`  Is scrollable: ${contentHeight.scrollable}`);
    console.log(`  Process rows: ${processRows}`);

    await page.screenshot({
      path: 'tests/e2e/screenshots/08-process-list-content-analysis.png',
      fullPage: true
    });
  });

  test('inspect computed CSS styles of panels', async ({ page }) => {
    const panels = await page.locator('[data-panel]').all();

    console.log('\n=== COMPUTED CSS INSPECTION ===\n');

    for (const panel of panels) {
      const panelName = await panel.getAttribute('data-panel');
      const isCollapsed = await panel.evaluate(el => el.classList.contains('collapsed'));

      const computed = await panel.evaluate(el => {
        const styles = window.getComputedStyle(el);
        return {
          flex: styles.flex,
          flexGrow: styles.flexGrow,
          flexShrink: styles.flexShrink,
          flexBasis: styles.flexBasis,
          height: styles.height,
          minHeight: styles.minHeight,
          maxHeight: styles.maxHeight,
          overflow: styles.overflow,
          display: styles.display,
          flexDirection: styles.flexDirection
        };
      });

      const box = await panel.boundingBox();

      console.log(`\nPanel: ${panelName}`);
      console.log(`  Collapsed: ${isCollapsed}`);
      console.log(`  Actual dimensions: ${box ? box.width : 0}x${box ? box.height : 0}`);
      console.log(`  Position: (${box ? box.x : 0}, ${box ? box.y : 0})`);
      console.log(`  Computed styles:`);
      console.log(`    flex: ${computed.flex}`);
      console.log(`    flex-grow: ${computed.flexGrow}`);
      console.log(`    flex-shrink: ${computed.flexShrink}`);
      console.log(`    flex-basis: ${computed.flexBasis}`);
      console.log(`    height: ${computed.height}`);
      console.log(`    min-height: ${computed.minHeight}`);
      console.log(`    max-height: ${computed.maxHeight}`);
      console.log(`    overflow: ${computed.overflow}`);
    }

    // Inspect terminal container
    const terminal = page.locator('.terminal-container');
    const terminalComputed = await terminal.evaluate(el => {
      const styles = window.getComputedStyle(el);
      return {
        flex: styles.flex,
        height: styles.height,
        maxHeight: styles.maxHeight,
        minHeight: styles.minHeight
      };
    });
    const terminalBox = await terminal.boundingBox();

    console.log(`\n.terminal-container:`);
    console.log(`  Actual dimensions: ${terminalBox ? terminalBox.width : 0}x${terminalBox ? terminalBox.height : 0}`);
    console.log(`  Position: (${terminalBox ? terminalBox.x : 0}, ${terminalBox ? terminalBox.y : 0})`);
    console.log(`  Computed styles:`);
    console.log(`    flex: ${terminalComputed.flex}`);
    console.log(`    height: ${terminalComputed.height}`);
    console.log(`    max-height: ${terminalComputed.maxHeight}`);
    console.log(`    min-height: ${terminalComputed.minHeight}`);

    // Inspect file-manager container
    const fileManager = page.locator('.file-manager');
    const fileManagerComputed = await fileManager.evaluate(el => {
      const styles = window.getComputedStyle(el);
      return {
        flex: styles.flex,
        height: styles.height,
        gap: styles.gap
      };
    });
    const fileManagerBox = await fileManager.boundingBox();

    console.log(`\n.file-manager:`);
    console.log(`  Actual dimensions: ${fileManagerBox ? fileManagerBox.width : 0}x${fileManagerBox ? fileManagerBox.height : 0}`);
    console.log(`  Position: (${fileManagerBox ? fileManagerBox.x : 0}, ${fileManagerBox ? fileManagerBox.y : 0})`);
    console.log(`  Computed styles:`);
    console.log(`    flex: ${fileManagerComputed.flex}`);
    console.log(`    height: ${fileManagerComputed.height}`);
    console.log(`    gap: ${fileManagerComputed.gap}`);

    // Also inspect the main container
    const main = page.locator('main');
    const mainComputed = await main.evaluate(el => {
      const styles = window.getComputedStyle(el);
      return {
        display: styles.display,
        flexDirection: styles.flexDirection,
        height: styles.height,
        minHeight: styles.minHeight,
        overflow: styles.overflow,
        gap: styles.gap
      };
    });

    const mainBox = await main.boundingBox();

    console.log(`\n<main> container:`);
    console.log(`  Actual dimensions: ${mainBox ? mainBox.width : 0}x${mainBox ? mainBox.height : 0}`);
    console.log(`  Computed styles:`);
    console.log(`    display: ${mainComputed.display}`);
    console.log(`    flex-direction: ${mainComputed.flexDirection}`);
    console.log(`    height: ${mainComputed.height}`);
    console.log(`    min-height: ${mainComputed.minHeight}`);
    console.log(`    overflow: ${mainComputed.overflow}`);
    console.log(`    gap: ${mainComputed.gap}`);

    await page.screenshot({
      path: 'tests/e2e/screenshots/07-computed-styles-inspection.png',
      fullPage: true
    });
  });
});
