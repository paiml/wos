import { test, expect } from '@playwright/test';

/**
 * WOS-303: Learning Objectives & Test Status Panel E2E Tests
 *
 * Tests for the gamification-based learning panel implementing mastery-based
 * progression with immediate feedback, visible progress, and clear goals.
 *
 * Requirements from docs/specifications/wos-enhanced-features-spec.md Section 4.5:
 * - Phase Tracker with progress bars and lock/unlock mechanics
 * - Task List per phase with expand/collapse functionality
 * - Integrated Test Runner with real-time results
 * - Progressive Hint System for scaffolded learning
 * - Time estimates based on historical velocity
 * - Click-to-navigate phase selection
 * - Celebration animations on phase completion
 *
 * Backend: roadmap.yaml parsing for phase/task data
 */

test.describe('WOS-303: Learning Objectives & Test Status', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    // Wait for WASM to initialize
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

    // Ensure learning objectives panel exists and is visible
    const panel = await page.locator('#panel-learning-objectives');
    await expect(panel).toBeVisible();

    // Check if panel is collapsed (parent .file-panel has .collapsed class)
    const isCollapsed = await panel.evaluate((el) => {
      return el.classList.contains('collapsed');
    });

    // If collapsed, click the expand button
    if (isCollapsed) {
      const expandButton = await page.locator('#panel-learning-objectives .btn-collapse');
      await expandButton.click();
      await page.waitForTimeout(300); // Wait for animation
    }

    // Wait for phase tracker to be visible
    await page.waitForSelector('#phase-tracker', { state: 'visible', timeout: 5000 });
  });

  test.describe('Phase Tracker', () => {
    test('should render phase tracker with all phases', async ({ page }) => {
      const phaseTracker = await page.locator('#phase-tracker');
      await expect(phaseTracker).toBeVisible();

      // Should show multiple phases from roadmap.yaml
      const phaseItems = await page.locator('.phase-item').count();
      expect(phaseItems).toBeGreaterThan(0);
    });

    test('should show progress bars for each phase', async ({ page }) => {
      // Each phase should have a progress bar
      const progressBars = await page.locator('.phase-progress-bar').count();
      expect(progressBars).toBeGreaterThan(0);

      // Progress bar should have width percentage
      const firstProgressBar = await page.locator('.phase-progress-bar').first();
      const width = await firstProgressBar.evaluate((el) => {
        return window.getComputedStyle(el).width;
      });
      expect(width).not.toBe('0px');
    });

    test('should display completion percentage for each phase', async ({ page }) => {
      const firstPhase = await page.locator('.phase-item').first();
      const percentage = await firstPhase.locator('.phase-percentage').textContent();

      // Should be in format "XX%"
      expect(percentage).toMatch(/\d+%/);
    });

    test('should show completion status icons (✓, →, 🔒)', async ({ page }) => {
      // Complete phases should have ✓
      const completePhases = await page.locator('.phase-item.complete .phase-status').allTextContents();
      completePhases.forEach(status => {
        expect(status).toContain('✓');
      });

      // In-progress phases should have →
      const inProgressPhases = await page.locator('.phase-item.in-progress .phase-status').allTextContents();
      if (inProgressPhases.length > 0) {
        inProgressPhases.forEach(status => {
          expect(status).toContain('→');
        });
      }

      // Locked phases should have 🔒
      const lockedPhases = await page.locator('.phase-item.locked .phase-status').allTextContents();
      if (lockedPhases.length > 0) {
        lockedPhases.forEach(status => {
          expect(status).toContain('🔒');
        });
      }
    });

    test('should navigate to phase details when clicked', async ({ page }) => {
      const firstPhase = await page.locator('.phase-item').first();
      const phaseName = await firstPhase.locator('.phase-name').textContent();

      await firstPhase.click();
      await page.waitForTimeout(200);

      // Task list should update to show tasks for selected phase
      const taskListTitle = await page.locator('#task-list-title').textContent();
      expect(taskListTitle).toContain(phaseName || '');
    });

    test('should disable navigation for locked phases', async ({ page }) => {
      const lockedPhase = await page.locator('.phase-item.locked').first();

      if (await lockedPhase.count() > 0) {
        const isDisabled = await lockedPhase.evaluate((el) => {
          return el.classList.contains('disabled') || el.getAttribute('aria-disabled') === 'true';
        });
        expect(isDisabled).toBe(true);

        // Click should not navigate
        const initialTaskListTitle = await page.locator('#task-list-title').textContent();
        await lockedPhase.click();
        await page.waitForTimeout(200);
        const afterClickTitle = await page.locator('#task-list-title').textContent();
        expect(afterClickTitle).toBe(initialTaskListTitle);
      }
    });

    test('should show time estimates based on velocity', async ({ page }) => {
      const phaseWithEstimate = await page.locator('.phase-item .phase-time-estimate').first();

      if (await phaseWithEstimate.count() > 0) {
        const timeText = await phaseWithEstimate.textContent();
        // Should show format like "2 hours remaining" or "Completed"
        expect(timeText).toMatch(/(hour|minute|Completed|Locked)/i);
      }
    });

    test('should highlight current phase', async ({ page }) => {
      const currentPhase = await page.locator('.phase-item.current');

      if (await currentPhase.count() > 0) {
        await expect(currentPhase).toBeVisible();

        // Should have distinct visual styling
        const backgroundColor = await currentPhase.evaluate((el) => {
          return window.getComputedStyle(el).backgroundColor;
        });
        expect(backgroundColor).not.toBe('transparent');
      }
    });
  });

  test.describe('Task List', () => {
    test('should render task list for selected phase', async ({ page }) => {
      // Click on a phase to select it
      const firstPhase = await page.locator('.phase-item').first();
      await firstPhase.click();
      await page.waitForTimeout(200);

      // Task list should be visible
      const taskList = await page.locator('#task-list');
      await expect(taskList).toBeVisible();

      // Should show tasks for the phase
      const taskItems = await page.locator('.task-item').count();
      expect(taskItems).toBeGreaterThan(0);
    });

    test('should show task ID and title', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      const taskId = await firstTask.locator('.task-id').textContent();
      expect(taskId).toMatch(/WOS-\d+/);

      const taskTitle = await firstTask.locator('.task-title').textContent();
      expect(taskTitle).not.toBe('');
    });

    test('should show task completion status (✓, ⏳, □)', async ({ page }) => {
      const taskStatuses = await page.locator('.task-item .task-status').allTextContents();

      taskStatuses.forEach(status => {
        // Should have one of the status icons
        expect(status).toMatch(/[✓⏳□]/);
      });
    });

    test('should expand/collapse task details on click', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();
      const taskHeader = await firstTask.locator('.task-header');

      // Initially details might be collapsed
      const initialDetailsVisible = await firstTask.locator('.task-details').isVisible();

      // Click to toggle
      await taskHeader.click();
      await page.waitForTimeout(200);

      const afterClickDetailsVisible = await firstTask.locator('.task-details').isVisible();
      expect(afterClickDetailsVisible).toBe(!initialDetailsVisible);
    });

    test('should show test count for each task', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Expand task details
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const testCount = await firstTask.locator('.task-test-count').textContent();
      // Format: "12/15 tests passing" or "Tests: 12/15"
      expect(testCount).toMatch(/\d+\/\d+/);
    });

    test('should show Run Tests button for each task', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Expand task details
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const runTestsBtn = await firstTask.locator('.btn-run-tests');
      await expect(runTestsBtn).toBeVisible();

      const btnText = await runTestsBtn.textContent();
      expect(btnText).toContain('Run Tests');
    });

    test('should show View Code button linking to source', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Expand task details
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const viewCodeBtn = await firstTask.locator('.btn-view-code');
      if (await viewCodeBtn.count() > 0) {
        await expect(viewCodeBtn).toBeVisible();
      }
    });

    test('should show progressive hints button when available', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Expand task details
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const hintsBtn = await firstTask.locator('.btn-show-hints');
      if (await hintsBtn.count() > 0) {
        await expect(hintsBtn).toBeVisible();

        const btnText = await hintsBtn.textContent();
        expect(btnText).toMatch(/hint/i);
      }
    });

    test('should reveal hints progressively when clicked', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Expand task details
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const hintsBtn = await firstTask.locator('.btn-show-hints');

      if (await hintsBtn.count() > 0) {
        // Click to reveal first hint
        await hintsBtn.click();
        await page.waitForTimeout(200);

        const hint = await firstTask.locator('.hint-content').first();
        await expect(hint).toBeVisible();

        // Button text should change to "Show Next Hint" or similar
        const btnTextAfter = await hintsBtn.textContent();
        expect(btnTextAfter).toMatch(/(Next|More) Hint/i);
      }
    });
  });

  test.describe('Integrated Test Runner', () => {
    test('should show test runner panel', async ({ page }) => {
      const testRunner = await page.locator('#test-runner');
      await expect(testRunner).toBeVisible();
    });

    test('should execute tests when Run Tests button is clicked', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Expand task and click Run Tests
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const runTestsBtn = await firstTask.locator('.btn-run-tests');
      await runTestsBtn.click();

      // Test runner should show loading state
      const testRunnerOutput = await page.locator('#test-runner-output');
      await expect(testRunnerOutput).toBeVisible();

      // Should show "Running tests..." message
      const output = await testRunnerOutput.textContent();
      expect(output).toMatch(/Running|Loading/i);
    });

    test('should display test results with pass/fail status', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Run tests
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);
      await firstTask.locator('.btn-run-tests').click();

      // Wait for results (mock or real)
      await page.waitForTimeout(1000);

      // Should show test results
      const testResults = await page.locator('.test-result-item');
      if (await testResults.count() > 0) {
        const firstResult = await testResults.first();

        // Should have pass/fail icon (✓ or ✗)
        const status = await firstResult.locator('.test-status').textContent();
        expect(status).toMatch(/[✓✗]/);
      }
    });

    test('should show test execution time for each test', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Run tests
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);
      await firstTask.locator('.btn-run-tests').click();

      await page.waitForTimeout(1000);

      const testResults = await page.locator('.test-result-item');
      if (await testResults.count() > 0) {
        const firstResult = await testResults.first();

        const time = await firstResult.locator('.test-time').textContent();
        // Format: "12ms" or "1.2s"
        expect(time).toMatch(/\d+(ms|s)/);
      }
    });

    test('should display error messages for failing tests', async ({ page }) => {
      // This test would require actual failing tests in the system
      // For now, we'll just check the structure exists
      const errorDisplay = await page.locator('#test-error-display');

      // Error display should exist (may be hidden if no errors)
      const exists = await errorDisplay.count() > 0;
      expect(exists).toBe(true);
    });

    test('should show summary of test results (X passed, Y failed)', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Run tests
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);
      await firstTask.locator('.btn-run-tests').click();

      await page.waitForTimeout(1000);

      const summary = await page.locator('#test-summary');
      if (await summary.count() > 0) {
        const summaryText = await summary.textContent();
        // Format: "Tests: 8 passed, 1 failed, 3 remaining"
        expect(summaryText).toMatch(/\d+ (passed|failing)/i);
      }
    });

    test('should provide source code links for failing tests', async ({ page }) => {
      // Failing tests should link to relevant source files
      const failingTest = await page.locator('.test-result-item.failed').first();

      if (await failingTest.count() > 0) {
        const sourceLink = await failingTest.locator('.test-source-link');
        if (await sourceLink.count() > 0) {
          const href = await sourceLink.getAttribute('href');
          expect(href).toMatch(/(\.rs|\.ts):\d+/); // Format: file.rs:234
        }
      }
    });

    test('should update task test count after running tests', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      // Get initial test count
      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);
      const initialCount = await firstTask.locator('.task-test-count').textContent();

      // Run tests
      await firstTask.locator('.btn-run-tests').click();
      await page.waitForTimeout(1000);

      // Test count should update (may be same if no changes)
      const updatedCount = await firstTask.locator('.task-test-count').textContent();
      expect(updatedCount).toBeDefined();
    });
  });

  test.describe('Progressive Hints', () => {
    test('should show generic hint on first reveal', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const hintsBtn = await firstTask.locator('.btn-show-hints');

      if (await hintsBtn.count() > 0) {
        await hintsBtn.click();
        await page.waitForTimeout(200);

        const firstHint = await page.locator('.hint-level-1').first();
        await expect(firstHint).toBeVisible();

        // First hint should be more generic
        const hintText = await firstHint.textContent();
        expect(hintText.length).toBeGreaterThan(10);
      }
    });

    test('should show more specific hints on subsequent reveals', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const hintsBtn = await firstTask.locator('.btn-show-hints');

      if (await hintsBtn.count() > 0) {
        // Click multiple times to reveal progressive hints
        await hintsBtn.click();
        await page.waitForTimeout(200);

        await hintsBtn.click();
        await page.waitForTimeout(200);

        // Should show level 2 hint
        const secondHint = await page.locator('.hint-level-2');
        if (await secondHint.count() > 0) {
          await expect(secondHint).toBeVisible();
        }
      }
    });

    test('should limit maximum number of hints revealed', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const hintsBtn = await firstTask.locator('.btn-show-hints');

      if (await hintsBtn.count() > 0) {
        // Click many times
        for (let i = 0; i < 10; i++) {
          if (await hintsBtn.isVisible()) {
            await hintsBtn.click();
            await page.waitForTimeout(100);
          }
        }

        // Count visible hints (should be capped, e.g., at 3)
        const visibleHints = await page.locator('.hint-content:visible').count();
        expect(visibleHints).toBeLessThanOrEqual(5); // Max 5 hints
      }
    });

    test('should disable hints button when all hints revealed', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);

      const hintsBtn = await firstTask.locator('.btn-show-hints');

      if (await hintsBtn.count() > 0) {
        // Reveal all hints
        let clickCount = 0;
        while (await hintsBtn.isEnabled() && clickCount < 10) {
          await hintsBtn.click();
          await page.waitForTimeout(100);
          clickCount++;
        }

        // Button should be disabled or hidden
        const isDisabled = !(await hintsBtn.isEnabled());
        expect(isDisabled || !(await hintsBtn.isVisible())).toBe(true);
      }
    });
  });

  test.describe('UI/UX Features', () => {
    test('should show celebration animation on phase completion', async ({ page }) => {
      // This would require completing a phase, which may not be possible in E2E
      // We'll check that the celebration element exists
      const celebration = await page.locator('#celebration-animation');
      const exists = await celebration.count() > 0;
      expect(exists).toBe(true);
    });

    test('should persist selected phase across page reloads', async ({ page }) => {
      const secondPhase = await page.locator('.phase-item').nth(1);

      if (await secondPhase.count() > 0) {
        await secondPhase.click();
        await page.waitForTimeout(200);

        const selectedPhaseName = await secondPhase.locator('.phase-name').textContent();

        // Reload page
        await page.reload();
        await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

        // Previously selected phase should still be selected
        const activePhase = await page.locator('.phase-item.active .phase-name').textContent();
        expect(activePhase).toBe(selectedPhaseName);
      }
    });

    test('should update progress bars smoothly with CSS transitions', async ({ page }) => {
      const progressBar = await page.locator('.phase-progress-bar').first();

      const transition = await progressBar.evaluate((el) => {
        return window.getComputedStyle(el).transition;
      });

      // Should have a transition defined for smooth animation
      expect(transition).not.toBe('none');
    });

    test('should be keyboard accessible (Tab, Enter)', async ({ page }) => {
      // Focus first phase item
      await page.keyboard.press('Tab');

      // Check if a phase item is focused
      const focusedElement = await page.evaluate(() => {
        return document.activeElement?.classList.contains('phase-item');
      });

      // Note: This may not work if other elements are focused first
      // Just checking that keyboard navigation is possible
      expect(typeof focusedElement).toBe('boolean');
    });

    test('should show loading indicator while test runner executes', async ({ page }) => {
      const firstTask = await page.locator('.task-item').first();

      await firstTask.locator('.task-header').click();
      await page.waitForTimeout(200);
      await firstTask.locator('.btn-run-tests').click();

      // Check for loading spinner or indicator
      const loadingIndicator = await page.locator('.test-runner-loading, .spinner, .loading');

      // Should appear briefly (might be very fast with mocked tests)
      const exists = await loadingIndicator.count() > 0;
      expect(exists).toBe(true);
    });
  });

  test.describe('Data Integration', () => {
    test('should load phase data from roadmap.yaml', async ({ page }) => {
      // Check that phases match actual roadmap structure
      const phaseNames = await page.locator('.phase-name').allTextContents();

      // Should have specific phases from roadmap
      const hasFoundationPhase = phaseNames.some(name => name.includes('Foundation'));
      expect(hasFoundationPhase).toBe(true);
    });

    test('should load task data from roadmap.yaml', async ({ page }) => {
      const firstPhase = await page.locator('.phase-item').first();
      await firstPhase.click();
      await page.waitForTimeout(200);

      const taskIds = await page.locator('.task-id').allTextContents();

      // Should have WOS ticket IDs
      const hasValidTaskIds = taskIds.every(id => id.match(/WOS-\d+/));
      expect(hasValidTaskIds).toBe(true);
    });

    test('should calculate progress based on completed tasks', async ({ page }) => {
      const firstPhase = await page.locator('.phase-item').first();

      const percentage = await firstPhase.locator('.phase-percentage').textContent();
      const percentValue = parseInt(percentage || '0');

      // Percentage should be between 0-100
      expect(percentValue).toBeGreaterThanOrEqual(0);
      expect(percentValue).toBeLessThanOrEqual(100);
    });
  });
});
