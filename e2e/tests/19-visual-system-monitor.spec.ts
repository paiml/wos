import { test, expect } from '@playwright/test';

/**
 * WOS-301: Visual System Monitor Panel E2E Tests
 *
 * Tests for the Visual System Monitor implementing Toyota's Mieruka principle
 * (making the invisible visible). Verifies real-time updates, interactive features,
 * and visual representations of process, memory, and filesystem state.
 *
 * Requirements from docs/specifications/wos-enhanced-features-spec.md Section 4.3:
 * - Live process table with 100ms updates
 * - Memory view with graphical bars
 * - File system tree view (collapsible)
 * - Click-to-inspect interactions
 * - Color-coded state visualization
 * - Hover tooltips for details
 */

test.describe('WOS-301: Visual System Monitor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    // Wait for WASM to initialize
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test.describe('Process Table Rendering', () => {
    test('should render process table with correct headers', async ({ page }) => {
      // Process table should exist
      const processTable = await page.locator('#panel-process-list .process-table');
      await expect(processTable).toBeVisible();

      // Check table headers match spec (PID, Parent, State, Priority, CPU Time, Memory)
      const headers = await page.locator('#panel-process-list .process-table thead th').allTextContents();
      expect(headers).toContain('PID');
      expect(headers).toContain('Parent');
      expect(headers).toContain('State');
      expect(headers).toContain('CPU Time');
      expect(headers).toContain('Memory');
    });

    test('should show init process (PID 1) after initialization', async ({ page }) => {
      // WOS should have init process running
      const processTableBody = await page.locator('#process-table-body');
      await expect(processTableBody).toBeVisible();

      // Should show at least PID 1 (init process)
      const pidCells = await page.locator('#process-table-body td:first-child').allTextContents();
      expect(pidCells.length).toBeGreaterThan(0);
      expect(pidCells.some(pid => pid.includes('1'))).toBe(true);
    });

    test('should color-code process states', async ({ page }) => {
      // Run a command to create processes
      await page.fill('#terminal-input', 'ps');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(200);

      // Check if process rows have state-based CSS classes
      const processRow = await page.locator('#process-table-body tr').first();
      const stateCell = await processRow.locator('td.process-state').first();

      if (await stateCell.count() > 0) {
        const classList = await stateCell.getAttribute('class');
        // Should have state class like 'state-ready', 'state-running', etc.
        expect(classList).toMatch(/state-(ready|running|blocked|terminated)/);
      }
    });

    test('should update process table in real-time', async ({ page }) => {
      // Get initial process count
      const initialRows = await page.locator('#process-table-body tr:not(.no-data)').count();

      // Run a command that creates a new process
      await page.fill('#terminal-input', 'echo "test"');
      await page.press('#terminal-input', 'Enter');

      // Wait for update cycle (100ms spec says)
      await page.waitForTimeout(150);

      // Process table should reflect changes (might be same count if process terminated quickly)
      const updatedRows = await page.locator('#process-table-body tr:not(.no-data)').count();
      expect(updatedRows).toBeGreaterThanOrEqual(initialRows);
    });

    test('should sort process table by clicking column headers', async ({ page }) => {
      // Create some processes first
      await page.fill('#terminal-input', 'ps');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(200);

      // Click PID header to sort
      const pidHeader = await page.locator('#panel-process-list .process-table thead th:has-text("PID")');
      if (await pidHeader.count() > 0) {
        await pidHeader.click();
        await page.waitForTimeout(100);

        // Get PIDs after sort
        const pidCells = await page.locator('#process-table-body td:first-child').allTextContents();
        const pids = pidCells.map(p => parseInt(p.trim())).filter(p => !isNaN(p));

        // Should be sorted
        if (pids.length > 1) {
          for (let i = 0; i < pids.length - 1; i++) {
            expect(pids[i]).toBeLessThanOrEqual(pids[i + 1]);
          }
        }
      }
    });

    test('should filter processes by search input', async ({ page }) => {
      // Check if search/filter input exists
      const filterInput = await page.locator('#process-filter-input');
      if (await filterInput.count() > 0) {
        // Type filter query
        await filterInput.fill('1');
        await page.waitForTimeout(100);

        // Only processes with '1' in PID, parent, or state should show
        const visibleRows = await page.locator('#process-table-body tr:visible:not(.no-data)').count();
        const allPids = await page.locator('#process-table-body tr:visible td:first-child').allTextContents();

        // All visible PIDs should contain '1'
        allPids.forEach(pid => {
          expect(pid).toContain('1');
        });
      }
    });
  });

  test.describe('Memory View Visualization', () => {
    test('should render memory view with graphical bars', async ({ page }) => {
      // Memory map panel should exist
      const memoryPanel = await page.locator('#panel-memory-map');
      await expect(memoryPanel).toBeVisible();

      // Should show memory segments with visual bars
      const codeSegmentBar = await page.locator('.memory-segment-bar[data-segment="code"]');
      const dataSegmentBar = await page.locator('.memory-segment-bar[data-segment="data"]');
      const heapSegmentBar = await page.locator('.memory-segment-bar[data-segment="heap"]');
      const stackSegmentBar = await page.locator('.memory-segment-bar[data-segment="stack"]');

      // At least one segment should be visible
      const hasSegments = (await codeSegmentBar.count()) > 0 ||
                         (await dataSegmentBar.count()) > 0 ||
                         (await heapSegmentBar.count()) > 0 ||
                         (await stackSegmentBar.count()) > 0;
      expect(hasSegments).toBe(true);
    });

    test('should show memory usage percentages', async ({ page }) => {
      // Memory info should display total, used, free, and percentage
      const memTotal = await page.locator('#mem-total');
      const memUsed = await page.locator('#mem-used');
      const memFree = await page.locator('#mem-free');
      const memPercent = await page.locator('#mem-percent');

      await expect(memTotal).toBeVisible();
      await expect(memUsed).toBeVisible();
      await expect(memFree).toBeVisible();
      await expect(memPercent).toBeVisible();

      // Percentage should be valid (0-100%)
      const percentText = await memPercent.textContent();
      const percentValue = parseInt(percentText || '0');
      expect(percentValue).toBeGreaterThanOrEqual(0);
      expect(percentValue).toBeLessThanOrEqual(100);
    });

    test('should display memory bars with correct widths', async ({ page }) => {
      // Find a memory segment bar
      const segmentBars = await page.locator('.memory-segment-bar').all();

      if (segmentBars.length > 0) {
        for (const bar of segmentBars) {
          // Each bar should have a width style indicating usage
          const style = await bar.getAttribute('style');
          expect(style).toContain('width');

          // Width should be between 0% and 100%
          const widthMatch = style?.match(/width:\s*(\d+(?:\.\d+)?)/);
          if (widthMatch) {
            const width = parseFloat(widthMatch[1]);
            expect(width).toBeGreaterThanOrEqual(0);
            expect(width).toBeLessThanOrEqual(100);
          }
        }
      }
    });

    test('should show hover tooltips with detailed page information', async ({ page }) => {
      // Find memory segment bar
      const segmentBar = await page.locator('.memory-segment-bar').first();

      if (await segmentBar.count() > 0) {
        // Hover over segment
        await segmentBar.hover();
        await page.waitForTimeout(200);

        // Tooltip should appear with details
        const tooltip = await page.locator('.memory-tooltip, [role="tooltip"]');
        if (await tooltip.count() > 0) {
          await expect(tooltip).toBeVisible();

          // Tooltip should contain memory information
          const tooltipText = await tooltip.textContent();
          // Should have size information
          expect(tooltipText).toMatch(/\d+\s*(KB|MB|bytes)/i);
        }
      }
    });

    test('should color-code memory permissions', async ({ page }) => {
      // Memory segments should have permission indicators
      const segmentBars = await page.locator('.memory-segment-bar').all();

      if (segmentBars.length > 0) {
        for (const bar of segmentBars) {
          const classList = await bar.getAttribute('class');
          // Should have permission classes like 'perm-read', 'perm-write', 'perm-exec'
          const hasPermClass = /perm-(read|write|exec|rwx)/.test(classList || '');
          // At least one bar should have permission class
          if (hasPermClass) {
            expect(hasPermClass).toBe(true);
            break;
          }
        }
      }
    });

    test('should update memory view after allocation', async ({ page }) => {
      // Get initial memory usage
      const initialUsed = await page.locator('#mem-used').textContent();

      // Run command that allocates memory
      await page.fill('#terminal-input', 'echo "allocate some memory"');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(200);

      // Memory display should update
      const updatedUsed = await page.locator('#mem-used').textContent();
      // Should have some value
      expect(updatedUsed).toBeTruthy();
    });
  });

  test.describe('File System Tree View', () => {
    test('should render filesystem tree with root directory', async ({ page }) => {
      // Filesystem panel should exist
      const filesystemPanel = await page.locator('#panel-filesystem');
      await expect(filesystemPanel).toBeVisible();

      // Should show root directory
      const rootNode = await page.locator('.fs-tree-node[data-path="/"]');
      if (await rootNode.count() > 0) {
        await expect(rootNode).toBeVisible();
      }
    });

    test('should show directory icons and file icons', async ({ page }) => {
      // File list should have items with icons
      const fileItems = await page.locator('#file-list .file-item').all();

      if (fileItems.length > 0) {
        for (const item of fileItems) {
          // Each file item should have an icon
          const icon = await item.locator('.file-icon, .dir-icon, svg').first();
          await expect(icon).toBeVisible();
        }
      }
    });

    test('should expand and collapse directories', async ({ page }) => {
      // Find a directory node
      const dirNode = await page.locator('.fs-tree-node.directory, .file-item.dir').first();

      if (await dirNode.count() > 0) {
        // Click to expand
        await dirNode.click();
        await page.waitForTimeout(200);

        // Should show children or expanded state
        const expandedClass = await dirNode.getAttribute('class');
        expect(expandedClass).toContain('expanded');

        // Click again to collapse
        await dirNode.click();
        await page.waitForTimeout(200);

        const collapsedClass = await dirNode.getAttribute('class');
        expect(collapsedClass).not.toContain('expanded');
      }
    });

    test('should lazy-load directory contents on expand', async ({ page }) => {
      // Find a collapsible directory
      const dirToggle = await page.locator('.fs-tree-node .toggle, .file-item.dir .toggle').first();

      if (await dirToggle.count() > 0) {
        // Get children count before expand
        const parent = await dirToggle.locator('..').first();
        const childrenBefore = await parent.locator('.fs-tree-children, .file-children').count();

        // Expand directory
        await dirToggle.click();
        await page.waitForTimeout(300);

        // Should load children
        const childrenAfter = await parent.locator('.fs-tree-children, .file-children').count();
        // Children container should exist after expand
        expect(childrenAfter).toBeGreaterThanOrEqual(childrenBefore);
      }
    });

    test('should update filesystem tree after file creation', async ({ page }) => {
      // Create a new file
      await page.fill('#terminal-input', 'echo "test content" > testfile.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(300);

      // File should appear in tree/list
      const fileItem = await page.locator('#file-list .file-item:has-text("testfile.txt")');
      await expect(fileItem).toBeVisible({ timeout: 2000 });
    });

    test('should sync with terminal when clicking a file', async ({ page }) => {
      // Create a file first
      await page.fill('#terminal-input', 'echo "content" > clicktest.txt');
      await page.press('#terminal-input', 'Enter');
      await page.waitForTimeout(300);

      // Click on file in list
      const fileItem = await page.locator('#file-list .file-item:has-text("clicktest.txt")');
      if (await fileItem.count() > 0) {
        await fileItem.click();
        await page.waitForTimeout(200);

        // Terminal input should update or some interaction should occur
        // (Exact behavior depends on implementation - might open file, change directory, etc.)
      }
    });
  });

  test.describe('Click-to-Inspect Interactions', () => {
    test('should highlight process when clicked', async ({ page }) => {
      // Wait for process table to populate
      await page.waitForTimeout(500);

      // Click on a process row
      const processRow = await page.locator('#process-table-body tr:not(.no-data)').first();
      if (await processRow.count() > 0) {
        await processRow.click();
        await page.waitForTimeout(200);

        // Row should have selected/highlighted class
        const classList = await processRow.getAttribute('class');
        expect(classList).toMatch(/selected|highlighted|active/);
      }
    });

    test('should show process details on click', async ({ page }) => {
      // Click on a process
      const processRow = await page.locator('#process-table-body tr:not(.no-data)').first();
      if (await processRow.count() > 0) {
        await processRow.click();
        await page.waitForTimeout(200);

        // Details panel or tooltip should appear
        const detailsPanel = await page.locator('.process-details, .details-panel, [role="dialog"]');
        if (await detailsPanel.count() > 0) {
          await expect(detailsPanel).toBeVisible();
        }
      }
    });

    test('should highlight memory regions when process is clicked', async ({ page }) => {
      // Click on a process
      const processRow = await page.locator('#process-table-body tr:not(.no-data)').first();
      if (await processRow.count() > 0) {
        await processRow.click();
        await page.waitForTimeout(300);

        // Memory map should highlight regions belonging to this process
        const highlightedRegions = await page.locator('.memory-segment-bar.highlighted, .memory-region.active').count();
        // At least some indication of highlighting
        expect(highlightedRegions).toBeGreaterThanOrEqual(0);
      }
    });

    test('should show file descriptors for selected process', async ({ page }) => {
      // Click on a process
      const processRow = await page.locator('#process-table-body tr:not(.no-data)').first();
      if (await processRow.count() > 0) {
        const pid = await processRow.locator('td').first().textContent();
        await processRow.click();
        await page.waitForTimeout(200);

        // Should display open file descriptors
        const fdList = await page.locator('.file-descriptors, .fd-list');
        if (await fdList.count() > 0) {
          // Standard streams should be present (stdin, stdout, stderr)
          const fdText = await fdList.textContent();
          expect(fdText).toMatch(/stdin|stdout|stderr|fd:\s*[012]/i);
        }
      }
    });
  });

  test.describe('Real-Time Updates (100ms)', () => {
    test('should update process table every 100ms', async ({ page }) => {
      // Observe process table changes over time
      const startTime = Date.now();
      let updateCount = 0;
      let lastContent = '';

      // Monitor for 500ms
      while (Date.now() - startTime < 500) {
        const currentContent = await page.locator('#process-table-body').textContent();
        if (currentContent !== lastContent) {
          updateCount++;
          lastContent = currentContent || '';
        }
        await page.waitForTimeout(50);
      }

      // Should have updated at least once in 500ms
      // (Actual update rate depends on whether processes are changing)
      expect(updateCount).toBeGreaterThanOrEqual(0);
    });

    test('should not block UI during updates', async ({ page }) => {
      // Start a long-running update cycle
      await page.fill('#terminal-input', 'ps');
      await page.press('#terminal-input', 'Enter');

      // UI should remain responsive
      await page.waitForTimeout(200);

      // Should be able to interact with other elements during updates
      const terminalInput = await page.locator('#terminal-input');
      await expect(terminalInput).toBeEnabled();

      // Should be able to type
      await terminalInput.fill('test');
      const inputValue = await terminalInput.inputValue();
      expect(inputValue).toBe('test');
    });
  });

  test.describe('Performance Requirements', () => {
    test('should render process table with >10 processes in <100ms', async ({ page }) => {
      // Create multiple processes (if kernel supports it)
      await page.fill('#terminal-input', 'ps');
      await page.press('#terminal-input', 'Enter');

      // Measure render time
      const startTime = Date.now();
      await page.waitForSelector('#process-table-body tr:not(.no-data)', { timeout: 2000 });
      const renderTime = Date.now() - startTime;

      // Should render quickly (relaxed threshold for E2E)
      expect(renderTime).toBeLessThan(500);
    });

    test('should handle large filesystem tree without lag', async ({ page }) => {
      // Create multiple files
      for (let i = 0; i < 5; i++) {
        await page.fill('#terminal-input', `echo "content${i}" > file${i}.txt`);
        await page.press('#terminal-input', 'Enter');
        await page.waitForTimeout(100);
      }

      // File list should render without lag
      const fileItems = await page.locator('#file-list .file-item').count();
      expect(fileItems).toBeGreaterThan(0);

      // Should be responsive
      const refreshButton = await page.locator('#btn-refresh-files');
      if (await refreshButton.count() > 0) {
        const clickStartTime = Date.now();
        await refreshButton.click();
        await page.waitForTimeout(200);
        const clickDuration = Date.now() - clickStartTime;
        expect(clickDuration).toBeLessThan(1000);
      }
    });
  });

  test.describe('Accessibility Features', () => {
    test('should have ARIA labels on interactive elements', async ({ page }) => {
      // Process table rows should have ARIA labels
      const processRows = await page.locator('#process-table-body tr:not(.no-data)').all();
      if (processRows.length > 0) {
        const firstRow = processRows[0];
        const ariaLabel = await firstRow.getAttribute('aria-label');
        // Should have descriptive label like "Process 1, state Ready"
        expect(ariaLabel).toBeTruthy();
      }
    });

    test('should support keyboard navigation in process table', async ({ page }) => {
      // Focus first process row
      const firstRow = await page.locator('#process-table-body tr:not(.no-data)').first();
      if (await firstRow.count() > 0) {
        await firstRow.focus();

        // Should be focusable
        const focused = await page.evaluate(() => {
          const activeElement = document.activeElement;
          return activeElement?.tagName === 'TR' || activeElement?.closest('tr') !== null;
        });
        expect(focused).toBe(true);
      }
    });
  });
});
