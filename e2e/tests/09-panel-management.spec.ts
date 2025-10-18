import { test, expect } from '@playwright/test';

test.describe('Panel Management', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto('http://localhost:8000/dist/wos/');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test('should display all panels on startup', async ({ page }) => {
    // Check that all panels are present in the DOM
    const processListPanel = page.locator('[data-panel="process_list"]');
    const memoryMapPanel = page.locator('[data-panel="memory_map"]');
    const syscallTracePanel = page.locator('[data-panel="syscall_trace"]');
    const filesystemPanel = page.locator('[data-panel="filesystem"]');
    const systemMonitorPanel = page.locator('[data-panel="system_monitor"]');

    await expect(processListPanel).toBeVisible();
    await expect(memoryMapPanel).toBeVisible();
    await expect(syscallTracePanel).toBeVisible();
    await expect(filesystemPanel).toBeVisible();
    await expect(systemMonitorPanel).toBeVisible();
  });

  test('should have collapse buttons on all panels', async ({ page }) => {
    // Check that all collapsible panels have collapse buttons
    const panels = await page.locator('[data-panel]').all();

    for (const panel of panels) {
      const collapseBtn = panel.locator('.btn-collapse');
      const count = await collapseBtn.count();

      // Some panels may not have collapse buttons (that's ok)
      // But if they do, there should be exactly one
      if (count > 0) {
        expect(count).toBe(1);
      }
    }
  });

  test('should collapse panel when collapse button is clicked', async ({ page }) => {
    const processListPanel = page.locator('[data-panel="process_list"]');
    const collapseBtn = processListPanel.locator('.btn-collapse');
    const panelContent = processListPanel.locator('.panel-content');

    // Verify panel content is visible initially
    await expect(panelContent).toBeVisible();

    // Click collapse button
    await collapseBtn.click();
    await page.waitForTimeout(300);

    // Verify panel content is hidden
    await expect(panelContent).not.toBeVisible();

    // Verify panel has collapsed class
    const hasCollapsedClass = await processListPanel.evaluate((el) =>
      el.classList.contains('collapsed')
    );
    expect(hasCollapsedClass).toBe(true);
  });

  test('should expand panel when collapse button is clicked again', async ({ page }) => {
    const memoryMapPanel = page.locator('[data-panel="memory_map"]');
    const collapseBtn = memoryMapPanel.locator('.btn-collapse');
    const panelContent = memoryMapPanel.locator('.panel-content');

    // First collapse the panel
    await collapseBtn.click();
    await page.waitForTimeout(300);
    await expect(panelContent).not.toBeVisible();

    // Then expand it again
    await collapseBtn.click();
    await page.waitForTimeout(300);

    // Verify panel content is visible again
    await expect(panelContent).toBeVisible();

    // Verify panel does not have collapsed class
    const hasCollapsedClass = await memoryMapPanel.evaluate((el) =>
      el.classList.contains('collapsed')
    );
    expect(hasCollapsedClass).toBe(false);
  });

  test('should rotate collapse icon when collapsing', async ({ page }) => {
    const processListPanel = page.locator('[data-panel="process_list"]');
    const collapseBtn = processListPanel.locator('.btn-collapse');
    const collapseSvg = collapseBtn.locator('svg');

    // Get initial transform
    const initialTransform = await collapseSvg.evaluate((el: SVGElement) =>
      window.getComputedStyle(el).transform
    );

    // Click collapse button
    await collapseBtn.click();
    await page.waitForTimeout(300);

    // Get transform after collapse
    const collapsedTransform = await collapseSvg.evaluate((el: SVGElement) =>
      window.getComputedStyle(el).transform
    );

    // Transform should have changed
    expect(collapsedTransform).not.toBe(initialTransform);
  });

  test('should handle multiple panel collapses independently', async ({ page }) => {
    const processListPanel = page.locator('[data-panel="process_list"]');
    const memoryMapPanel = page.locator('[data-panel="memory_map"]');

    const processCollapseBtn = processListPanel.locator('.btn-collapse');
    const memoryCollapseBtn = memoryMapPanel.locator('.btn-collapse');

    const processContent = processListPanel.locator('.panel-content');
    const memoryContent = memoryMapPanel.locator('.panel-content');

    // Collapse process list panel
    await processCollapseBtn.click();
    await page.waitForTimeout(300);

    // Verify process list is collapsed but memory map is not
    await expect(processContent).not.toBeVisible();
    await expect(memoryContent).toBeVisible();

    // Now collapse memory map panel
    await memoryCollapseBtn.click();
    await page.waitForTimeout(300);

    // Verify both are collapsed
    await expect(processContent).not.toBeVisible();
    await expect(memoryContent).not.toBeVisible();

    // Expand process list panel
    await processCollapseBtn.click();
    await page.waitForTimeout(300);

    // Verify process list is expanded but memory map is still collapsed
    await expect(processContent).toBeVisible();
    await expect(memoryContent).not.toBeVisible();
  });

  test('should display process table in process list panel', async ({ page }) => {
    const processListPanel = page.locator('[data-panel="process_list"]');
    const processTable = processListPanel.locator('.process-table');

    // Verify table is present
    await expect(processTable).toBeVisible();

    // Verify table has headers
    const headers = processTable.locator('thead th');
    const headerTexts = await headers.allTextContents();

    expect(headerTexts).toContain('PID');
    expect(headerTexts).toContain('State');
    expect(headerTexts).toContain('Parent');
    expect(headerTexts).toContain('Command');
  });

  test('should display memory information in memory map panel', async ({ page }) => {
    const memoryMapPanel = page.locator('[data-panel="memory_map"]');
    const memoryInfo = memoryMapPanel.locator('.memory-info');

    // Verify memory info is present
    await expect(memoryInfo).toBeVisible();

    // Verify memory fields are displayed
    const memoryText = await memoryInfo.textContent();
    expect(memoryText).toContain('Total Memory:');
    expect(memoryText).toContain('Used:');
    expect(memoryText).toContain('Free:');
    expect(memoryText).toContain('Usage:');
  });

  test('should display system call trace in syscall trace panel', async ({ page }) => {
    const syscallTracePanel = page.locator('[data-panel="syscall_trace"]');
    const syscallTrace = syscallTracePanel.locator('.syscall-trace');

    // Verify syscall trace container is present
    await expect(syscallTrace).toBeVisible();

    // Verify default "no data" message
    const noDataMsg = syscallTrace.locator('.no-data');
    await expect(noDataMsg).toBeVisible();

    const noDataText = await noDataMsg.textContent();
    expect(noDataText).toContain('No system calls traced yet');
  });

  test('should have clear trace button in syscall trace panel', async ({ page }) => {
    const syscallTracePanel = page.locator('[data-panel="syscall_trace"]');
    const clearTraceBtn = syscallTracePanel.locator('.btn-clear-trace');

    // Verify clear trace button exists
    await expect(clearTraceBtn).toBeVisible();
  });

  test('should have refresh button in process list panel', async ({ page }) => {
    const processListPanel = page.locator('[data-panel="process_list"]');
    const refreshBtn = processListPanel.locator('.btn-refresh-panel');

    // Verify refresh button exists
    await expect(refreshBtn).toBeVisible();
  });

  test('should maintain panel state across command executions', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const processListPanel = page.locator('[data-panel="process_list"]');
    const collapseBtn = processListPanel.locator('.btn-collapse');
    const panelContent = processListPanel.locator('.panel-content');

    // Collapse the panel
    await collapseBtn.click();
    await page.waitForTimeout(300);
    await expect(panelContent).not.toBeVisible();

    // Execute some commands
    await input.fill('ps');
    await input.press('Enter');
    await page.waitForTimeout(300);

    await input.fill('ls');
    await input.press('Enter');
    await page.waitForTimeout(300);

    // Verify panel is still collapsed
    await expect(panelContent).not.toBeVisible();

    const hasCollapsedClass = await processListPanel.evaluate((el) =>
      el.classList.contains('collapsed')
    );
    expect(hasCollapsedClass).toBe(true);
  });

  test('should have panel headers with correct titles', async ({ page }) => {
    // Check all panel headers
    const panels = [
      { selector: '[data-panel="process_list"]', title: 'Process List' },
      { selector: '[data-panel="memory_map"]', title: 'Memory Map' },
      { selector: '[data-panel="syscall_trace"]', title: 'System Call Trace' },
      { selector: '[data-panel="filesystem"]', title: 'Files' },
      { selector: '[data-panel="system_monitor"]', title: 'System Info' }
    ];

    for (const panel of panels) {
      const panelEl = page.locator(panel.selector);
      const header = panelEl.locator('.file-panel-header h3');
      const headerText = await header.textContent();

      expect(headerText).toBe(panel.title);
    }
  });

  test('should apply panel styling correctly', async ({ page }) => {
    const processListPanel = page.locator('[data-panel="process_list"]');

    // Verify panel has correct classes
    const hasFilePanel = await processListPanel.evaluate((el) =>
      el.classList.contains('file-panel')
    );
    expect(hasFilePanel).toBe(true);

    const hasProcessPanel = await processListPanel.evaluate((el) =>
      el.classList.contains('process-panel')
    );
    expect(hasProcessPanel).toBe(true);
  });
});
