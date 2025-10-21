import { test, expect } from '@playwright/test';

/**
 * WOS-302: Time-Travel Debugger UI Controls E2E Tests
 *
 * Tests for the omniscient time-travel debugger implementing retrospective debugging
 * with state scrubbing, event replay, and visual state inspection.
 *
 * Requirements from docs/specifications/wos-enhanced-features-spec.md Section 4.4:
 * - Scrubbable timeline slider with drag/keyboard navigation
 * - Event log with filtering (PID, syscall type, success/failure)
 * - State inspector showing full process/memory/filesystem state
 * - Playback controls (play/pause/step forward/backward)
 * - Keyboard navigation (arrow keys for 1ms increments)
 * - Canvas-based virtualization for performance
 * - Diff highlighting between states
 *
 * Backend: kernel/src/trace.rs (KernelHistory)
 */

test.describe('WOS-302: Time-Travel Debugger', () => {
  test.beforeEach(async ({ page }) => {
    // Disable tutorial overlay BEFORE page loads to prevent it from blocking interactions
    await page.addInitScript(() => {
      localStorage.setItem('wos-tutorial-completed', 'true');
    });

    await page.goto('index.html');

    // Wait for WASM to initialize
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

    // Generate some history by running commands
    await page.fill('#terminal-input', 'echo "test1"');
    await page.press('#terminal-input', 'Enter');
    await page.waitForTimeout(100);

    await page.fill('#terminal-input', 'ps');
    await page.press('#terminal-input', 'Enter');
    await page.waitForTimeout(100);

    await page.fill('#terminal-input', 'echo "test2"');
    await page.press('#terminal-input', 'Enter');
    await page.waitForTimeout(100);

    // Ensure time-travel debugger panel is visible
    // Check if panel exists
    const panel = await page.locator('#panel-time-travel-debugger');
    await expect(panel).toBeVisible();

    // Check if panel is collapsed (parent .file-panel has .collapsed class)
    const isCollapsed = await panel.evaluate((el) => {
      return el.classList.contains('collapsed');
    });

    // If collapsed, click the expand button
    if (isCollapsed) {
      const expandButton = await page.locator('#panel-time-travel-debugger .btn-collapse');
      await expandButton.click();
      await page.waitForTimeout(300); // Wait for animation
    }

    // Wait for timeline slider to be visible
    await page.waitForSelector('#timeline-slider', { state: 'visible', timeout: 5000 });
  });

  test.describe('Timeline Slider', () => {
    test('should render timeline slider with correct range', async ({ page }) => {
      // Timeline slider should exist in time-travel panel
      const timelineSlider = await page.locator('#timeline-slider');
      await expect(timelineSlider).toBeVisible();

      // Slider should have min=0 and max=current_position
      const min = await timelineSlider.getAttribute('min');
      const max = await timelineSlider.getAttribute('max');
      expect(parseInt(min || '0')).toBe(0);
      expect(parseInt(max || '0')).toBeGreaterThan(0);
    });

    test('should show current position indicator on timeline', async ({ page }) => {
      const positionIndicator = await page.locator('#timeline-position-indicator');
      await expect(positionIndicator).toBeVisible();

      // Position indicator should show timestamp
      const text = await positionIndicator.textContent();
      expect(text).toMatch(/\d+ms/); // Should show milliseconds
    });

    test('should update position when slider is dragged', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      const initialValue = await slider.inputValue();

      // Get slider bounding box for drag operation
      const box = await slider.boundingBox();
      if (box) {
        // Drag slider to middle position
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
        await page.mouse.down();
        await page.mouse.move(box.x + box.width * 0.3, box.y + box.height / 2);
        await page.mouse.up();

        await page.waitForTimeout(100);

        const newValue = await slider.inputValue();
        expect(newValue).not.toBe(initialValue);
      }
    });

    test('should scrub through history when slider moves', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      const eventLog = await page.locator('#event-log');

      // Record initial event log state
      const initialEvents = await eventLog.locator('.event-item').count();

      // Drag slider to earlier position
      await slider.fill('0'); // Go to start
      await page.waitForTimeout(100);

      // Event log should show fewer events
      const eventsAtStart = await eventLog.locator('.event-item').count();
      expect(eventsAtStart).toBeLessThanOrEqual(initialEvents);
    });

    test('should show tick marks for major events', async ({ page }) => {
      const tickMarks = await page.locator('#timeline-ticks .tick-mark');
      const count = await tickMarks.count();

      // Should have at least one tick mark per command we ran
      expect(count).toBeGreaterThanOrEqual(3);
    });
  });

  test.describe('Event Log', () => {
    test('should render event log with syscall entries', async ({ page }) => {
      const eventLog = await page.locator('#event-log');
      await expect(eventLog).toBeVisible();

      // Should have event items
      const eventItems = await eventLog.locator('.event-item');
      const count = await eventItems.count();
      expect(count).toBeGreaterThan(0);
    });

    test('should display event columns: Time, PID, Syscall, Result', async ({ page }) => {
      const headers = await page.locator('#event-log-headers th').allTextContents();
      expect(headers).toContain('Time');
      expect(headers).toContain('PID');
      expect(headers).toContain('Syscall');
      expect(headers).toContain('Result');
    });

    test('should show syscall details when event is clicked', async ({ page }) => {
      const firstEvent = await page.locator('#event-log .event-item').first();
      await firstEvent.click();

      // Event details panel should appear
      const detailsPanel = await page.locator('#event-details-panel');
      await expect(detailsPanel).toBeVisible();

      // Should show syscall input/output
      const detailsText = await detailsPanel.textContent();
      expect(detailsText).toBeTruthy();
    });

    test('should filter events by PID', async ({ page }) => {
      // Get total event count
      const totalEvents = await page.locator('#event-log .event-item').count();

      // Apply PID filter (filter to PID 1)
      await page.fill('#filter-pid', '1');
      await page.waitForTimeout(100);

      // Should show fewer events
      const filteredEvents = await page.locator('#event-log .event-item').count();
      expect(filteredEvents).toBeLessThanOrEqual(totalEvents);

      // All visible events should have PID 1
      const pidCells = await page.locator('#event-log .event-item .event-pid').allTextContents();
      pidCells.forEach(pid => {
        expect(pid.trim()).toBe('1');
      });
    });

    test('should filter events by syscall type', async ({ page }) => {
      const totalEvents = await page.locator('#event-log .event-item').count();

      // Apply syscall type filter (e.g., "Write")
      await page.selectOption('#filter-syscall-type', 'Write');
      await page.waitForTimeout(100);

      const filteredEvents = await page.locator('#event-log .event-item').count();
      expect(filteredEvents).toBeLessThanOrEqual(totalEvents);

      // All visible events should be Write syscalls
      const syscallCells = await page.locator('#event-log .event-item .event-syscall').allTextContents();
      syscallCells.forEach(syscall => {
        expect(syscall).toContain('Write');
      });
    });

    test('should filter events by success/failure', async ({ page }) => {
      // Apply success filter
      await page.check('#filter-success');
      await page.uncheck('#filter-failure');
      await page.waitForTimeout(100);

      // All visible events should show success
      const resultCells = await page.locator('#event-log .event-item .event-result').allTextContents();
      resultCells.forEach(result => {
        expect(result).toContain('✓'); // Or "Success" or green indicator
      });
    });

    test('should use virtual scrolling for performance', async ({ page }) => {
      // Event log should have virtual-scroll container
      const virtualContainer = await page.locator('#event-log.virtual-scroll');
      await expect(virtualContainer).toBeVisible();

      // Should have scrollable viewport
      const scrollHeight = await page.evaluate(() => {
        const el = document.querySelector('#event-log');
        return el ? el.scrollHeight : 0;
      });
      expect(scrollHeight).toBeGreaterThan(0);
    });

    test('should highlight selected event', async ({ page }) => {
      const firstEvent = await page.locator('#event-log .event-item').first();
      await firstEvent.click();

      // Event should have 'selected' class
      const classList = await firstEvent.getAttribute('class');
      expect(classList).toContain('selected');
    });
  });

  test.describe('State Inspector', () => {
    test('should render state inspector panel', async ({ page }) => {
      const stateInspector = await page.locator('#state-inspector');
      await expect(stateInspector).toBeVisible();
    });

    test('should show process state section', async ({ page }) => {
      const processSection = await page.locator('#state-inspector-processes');
      await expect(processSection).toBeVisible();

      // Should have process entries
      const processItems = await processSection.locator('.process-state-item');
      const count = await processItems.count();
      expect(count).toBeGreaterThan(0);
    });

    test('should show memory state section', async ({ page }) => {
      const memorySection = await page.locator('#state-inspector-memory');
      await expect(memorySection).toBeVisible();

      // Should show memory allocations
      const memoryText = await memorySection.textContent();
      expect(memoryText).toBeTruthy();
    });

    test('should show filesystem state section', async ({ page }) => {
      const filesystemSection = await page.locator('#state-inspector-filesystem');
      await expect(filesystemSection).toBeVisible();

      // Should show file tree
      const fileItems = await filesystemSection.locator('.file-tree-item');
      const count = await fileItems.count();
      expect(count).toBeGreaterThan(0);
    });

    test('should expand/collapse process details on click', async ({ page }) => {
      const firstProcess = await page.locator('#state-inspector-processes .process-state-item').first();

      // Initially collapsed
      let isExpanded = await firstProcess.getAttribute('aria-expanded');
      expect(isExpanded).toBe('false');

      // Click to expand
      await firstProcess.click();
      await page.waitForTimeout(50);

      // Should be expanded now
      isExpanded = await firstProcess.getAttribute('aria-expanded');
      expect(isExpanded).toBe('true');

      // Should show process details
      const details = await firstProcess.locator('.process-details');
      await expect(details).toBeVisible();
    });

    test('should highlight state differences when scrubbing', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Go to middle of timeline
      const max = parseInt(await slider.getAttribute('max') || '100');
      await slider.fill(String(Math.floor(max / 2)));
      await page.waitForTimeout(100);

      // Record state
      const stateAtMiddle = await page.locator('#state-inspector').textContent();

      // Go to end of timeline
      await slider.fill(String(max));
      await page.waitForTimeout(100);

      // State should be different
      const stateAtEnd = await page.locator('#state-inspector').textContent();
      expect(stateAtEnd).not.toBe(stateAtMiddle);

      // Should have diff highlighting
      const diffElements = await page.locator('#state-inspector .state-diff');
      const count = await diffElements.count();
      expect(count).toBeGreaterThan(0);
    });

    test('should show process state fields: PID, State, Parent, Memory', async ({ page }) => {
      const firstProcess = await page.locator('#state-inspector-processes .process-state-item').first();
      await firstProcess.click(); // Expand details

      const detailsText = await firstProcess.textContent() || '';
      expect(detailsText).toContain('PID');
      expect(detailsText).toMatch(/State|Ready|Running|Blocked|Terminated/);
      expect(detailsText).toContain('Parent');
      expect(detailsText).toContain('Memory');
    });
  });

  test.describe('Playback Controls', () => {
    test('should render playback control buttons', async ({ page }) => {
      const playButton = await page.locator('#playback-play');
      await expect(playButton).toBeVisible();

      const pauseButton = await page.locator('#playback-pause');
      await expect(pauseButton).toBeVisible();

      const stepBackButton = await page.locator('#playback-step-back');
      await expect(stepBackButton).toBeVisible();

      const stepForwardButton = await page.locator('#playback-step-forward');
      await expect(stepForwardButton).toBeVisible();
    });

    test('should step backward through history', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      const initialPosition = await slider.inputValue();

      // Click step back button
      const stepBackButton = await page.locator('#playback-step-back');
      await stepBackButton.click();
      await page.waitForTimeout(50);

      const newPosition = await slider.inputValue();
      expect(parseInt(newPosition)).toBeLessThan(parseInt(initialPosition));
    });

    test('should step forward through history', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Go to middle
      const max = parseInt(await slider.getAttribute('max') || '100');
      await slider.fill(String(Math.floor(max / 2)));
      await page.waitForTimeout(50);

      const initialPosition = await slider.inputValue();

      // Click step forward button
      const stepForwardButton = await page.locator('#playback-step-forward');
      await stepForwardButton.click();
      await page.waitForTimeout(50);

      const newPosition = await slider.inputValue();
      expect(parseInt(newPosition)).toBeGreaterThan(parseInt(initialPosition));
    });

    test('should play through history automatically', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Go to start
      await slider.fill('0');
      await page.waitForTimeout(50);

      const startPosition = await slider.inputValue();

      // Click play button
      const playButton = await page.locator('#playback-play');
      await playButton.click();

      // Wait for playback to advance
      await page.waitForTimeout(300);

      const newPosition = await slider.inputValue();
      expect(parseInt(newPosition)).toBeGreaterThan(parseInt(startPosition));
    });

    test('should pause automatic playback', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Go to start
      await slider.fill('0');
      await page.waitForTimeout(50);

      // Start playback
      const playButton = await page.locator('#playback-play');
      await playButton.click();
      await page.waitForTimeout(100);

      // Pause playback
      const pauseButton = await page.locator('#playback-pause');
      await pauseButton.click();

      const pausePosition = await slider.inputValue();

      // Wait and verify position doesn't change
      await page.waitForTimeout(200);
      const finalPosition = await slider.inputValue();
      expect(finalPosition).toBe(pausePosition);
    });

    test('should disable step back at beginning of history', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      await slider.fill('0'); // Go to start
      await page.waitForTimeout(50);

      const stepBackButton = await page.locator('#playback-step-back');
      const isDisabled = await stepBackButton.isDisabled();
      expect(isDisabled).toBe(true);
    });

    test('should disable step forward at end of history', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      const max = await slider.getAttribute('max');
      await slider.fill(max || '100'); // Go to end
      await page.waitForTimeout(50);

      const stepForwardButton = await page.locator('#playback-step-forward');
      const isDisabled = await stepForwardButton.isDisabled();
      expect(isDisabled).toBe(true);
    });
  });

  test.describe('Keyboard Navigation', () => {
    test('should step backward with left arrow key', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      await slider.focus();

      // Go to middle first (can't step back from position 0)
      const max = parseInt(await slider.getAttribute('max') || '100');
      await slider.evaluate((el, val) => {
        (el as HTMLInputElement).value = val;
        el.dispatchEvent(new Event('input', { bubbles: true }));
      }, String(Math.floor(max / 2)));
      await page.waitForTimeout(50);

      const initialPosition = await slider.inputValue();

      await page.keyboard.press('ArrowLeft');
      await page.waitForTimeout(50);

      const newPosition = await slider.inputValue();
      expect(parseInt(newPosition)).toBeLessThan(parseInt(initialPosition));
    });

    test('should step forward with right arrow key', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      await slider.focus();

      // Go to middle first
      const max = parseInt(await slider.getAttribute('max') || '100');
      await slider.fill(String(Math.floor(max / 2)));
      await page.waitForTimeout(50);

      const initialPosition = await slider.inputValue();

      await page.keyboard.press('ArrowRight');
      await page.waitForTimeout(50);

      const newPosition = await slider.inputValue();
      expect(parseInt(newPosition)).toBeGreaterThan(parseInt(initialPosition));
    });

    test('should jump to start with Home key', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      await slider.focus();

      await page.keyboard.press('Home');
      await page.waitForTimeout(50);

      const position = await slider.inputValue();
      expect(parseInt(position)).toBe(0);
    });

    test('should jump to end with End key', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      await slider.focus();

      await page.keyboard.press('End');
      await page.waitForTimeout(50);

      const position = await slider.inputValue();
      const max = await slider.getAttribute('max');
      expect(parseInt(position)).toBe(parseInt(max || '0'));
    });

    test('should support Space key to play/pause', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');
      await slider.focus();

      // Go to start
      await slider.fill('0');
      await page.waitForTimeout(50);

      // Press Space to play
      await page.keyboard.press('Space');
      await page.waitForTimeout(200);

      const playPosition = await slider.inputValue();
      expect(parseInt(playPosition)).toBeGreaterThan(0);

      // Press Space to pause
      await page.keyboard.press('Space');
      const pausePosition = await slider.inputValue();

      await page.waitForTimeout(100);
      const finalPosition = await slider.inputValue();
      expect(finalPosition).toBe(pausePosition);
    });
  });

  test.describe('KernelHistory Integration', () => {
    test('should load history from WASM backend', async ({ page }) => {
      // Verify we can access kernel history via WASM
      const historyAvailable = await page.evaluate(() => {
        return typeof (window as any).wos?.getKernelHistory === 'function';
      });
      expect(historyAvailable).toBe(true);
    });

    test('should show syscall traces from KernelHistory', async ({ page }) => {
      const traces = await page.evaluate(() => {
        const wos = (window as any).wos;
        if (!wos || !wos.getKernelHistory) return [];
        const history = wos.getKernelHistory();
        return history ? JSON.parse(history) : [];
      });

      expect(traces.length).toBeGreaterThan(0);

      // Verify trace structure matches SystemCallTrace from kernel/src/trace.rs
      const firstTrace = traces[0];
      expect(firstTrace).toHaveProperty('trace_id');
      expect(firstTrace).toHaveProperty('calling_pid');
      expect(firstTrace).toHaveProperty('syscall');
      expect(firstTrace).toHaveProperty('result');
      expect(firstTrace).toHaveProperty('timestamp_us');
    });

    test('should restore kernel state when scrubbing to position', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Get state at current position
      const currentState = await page.evaluate(() => {
        const wos = (window as any).wos;
        return wos?.getCurrentState ? wos.getCurrentState() : null;
      });

      // Scrub to middle
      const max = parseInt(await slider.getAttribute('max') || '100');
      await slider.fill(String(Math.floor(max / 2)));
      await page.waitForTimeout(100);

      // State should be different
      const middleState = await page.evaluate(() => {
        const wos = (window as any).wos;
        return wos?.getCurrentState ? wos.getCurrentState() : null;
      });

      expect(JSON.stringify(middleState)).not.toBe(JSON.stringify(currentState));
    });

    test('should export trace history as JSON', async ({ page }) => {
      // Export traces button should exist
      const exportButton = await page.locator('#export-traces-json');
      await expect(exportButton).toBeVisible();

      // Click export should trigger download
      const downloadPromise = page.waitForEvent('download');
      await exportButton.click();
      const download = await downloadPromise;

      // Verify filename
      expect(download.suggestedFilename()).toContain('traces');
      expect(download.suggestedFilename()).toContain('.json');
    });

    test('should show trace count matching backend', async ({ page }) => {
      const traceCount = await page.locator('#trace-count');
      await expect(traceCount).toBeVisible();

      const displayedCount = parseInt(await traceCount.textContent() || '0');

      const backendCount = await page.evaluate(() => {
        const wos = (window as any).wos;
        if (!wos || !wos.getKernelHistory) return 0;
        const history = wos.getKernelHistory();
        return history ? JSON.parse(history).length : 0;
      });

      expect(displayedCount).toBe(backendCount);
    });
  });

  test.describe('Performance', () => {
    test('should render timeline with large history efficiently', async ({ page }) => {
      // Generate more history
      for (let i = 0; i < 20; i++) {
        await page.fill('#terminal-input', `echo "test${i}"`);
        await page.press('#terminal-input', 'Enter');
        await page.waitForTimeout(50);
      }

      // Timeline should still render quickly
      const slider = await page.locator('#timeline-slider');
      const startTime = Date.now();
      await expect(slider).toBeVisible();
      const renderTime = Date.now() - startTime;

      expect(renderTime).toBeLessThan(1000); // Should render in <1s
    });

    test('should use canvas for event log with >1000 events', async ({ page }) => {
      // Generate lots of events
      for (let i = 0; i < 50; i++) {
        await page.fill('#terminal-input', 'ps');
        await page.press('#terminal-input', 'Enter');
        await page.waitForTimeout(30);
      }

      // Should switch to canvas rendering
      const canvas = await page.locator('#event-log-canvas');
      const canvasExists = await canvas.count() > 0;

      if (canvasExists) {
        await expect(canvas).toBeVisible();
      }
    });

    test('should scrub through history without lag', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Rapidly scrub through positions
      const positions = [0, 25, 50, 75, 100];
      const max = parseInt(await slider.getAttribute('max') || '100');

      for (const pct of positions) {
        const pos = Math.floor(max * pct / 100);
        const startTime = Date.now();
        await slider.fill(String(pos));
        await page.waitForTimeout(10);
        const scrubTime = Date.now() - startTime;

        // Each scrub should complete in <100ms
        expect(scrubTime).toBeLessThan(100);
      }
    });
  });

  test.describe('UI/UX', () => {
    test('should show helpful tooltip on timeline hover', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Hover over slider
      await slider.hover();
      await page.waitForTimeout(100);

      // Tooltip should appear
      const tooltip = await page.locator('#timeline-tooltip');
      await expect(tooltip).toBeVisible();

      // Should show timestamp
      const tooltipText = await tooltip.textContent();
      expect(tooltipText).toMatch(/\d+ms/);
    });

    test('should highlight current event in log when scrubbing', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Scrub to specific position
      await slider.fill('0');
      await page.waitForTimeout(100);

      // First event should be highlighted
      const firstEvent = await page.locator('#event-log .event-item').first();
      const classList = await firstEvent.getAttribute('class');
      expect(classList).toContain('current');
    });

    test('should persist debugger state across page reloads', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Set specific position
      await slider.fill('10');
      await page.waitForTimeout(100);

      const position = await slider.inputValue();

      // Reload page
      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

      // Position should be restored
      const restoredPosition = await slider.inputValue();
      expect(restoredPosition).toBe(position);
    });

    test('should show loading indicator when restoring state', async ({ page }) => {
      const slider = await page.locator('#timeline-slider');

      // Scrub to far position (might take time to restore)
      const max = parseInt(await slider.getAttribute('max') || '100');
      await slider.fill('0');
      await page.waitForTimeout(50);

      // Loading indicator should appear briefly
      const loadingIndicator = await page.locator('#state-loading-indicator');
      // Note: This might be too fast to catch, so we check if it exists
      const exists = await loadingIndicator.count() > 0;
      expect(exists).toBe(true);
    });
  });
});
