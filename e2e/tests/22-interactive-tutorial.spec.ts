/**
 * WOS-304: Interactive Tutorial (First-Run Experience)
 *
 * E2E tests for 5-minute guided walkthrough showing:
 * - 6-step tutorial flow (Welcome → Terminal → Monitor → Debugger → Tests → Completion)
 * - UI element highlighting with glowing borders
 * - Skip tutorial functionality
 * - Retake tutorial from help menu
 * - localStorage persistence of tutorial state
 *
 * Based on UX research: Interactive onboarding reduces abandonment
 * (Nielsen, 2000; Harrison et al., 2018)
 */

import { test, expect } from '@playwright/test';

test.describe('WOS-304: Interactive Tutorial', () => {
  test.beforeEach(async ({ page }) => {
    // Clear tutorial state for fresh start
    await page.goto('index.html');
    await page.evaluate(() => {
      localStorage.removeItem('wos-tutorial-completed');
      localStorage.removeItem('wos-tutorial-step');
    });

    // Wait for app initialization
    await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });
  });

  test.describe('Tutorial Initialization', () => {
    test('should show tutorial overlay on first visit', async ({ page }) => {
      // Reload to trigger first-visit check
      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

      const tutorialOverlay = await page.locator('#tutorial-overlay');
      await expect(tutorialOverlay).toBeVisible();
    });

    test('should not show tutorial on subsequent visits', async ({ page }) => {
      // Mark tutorial as completed
      await page.evaluate(() => {
        localStorage.setItem('wos-tutorial-completed', 'true');
      });

      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

      const tutorialOverlay = await page.locator('#tutorial-overlay');
      await expect(tutorialOverlay).not.toBeVisible();
    });

    test('should have accessibility attributes', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

      const tutorialOverlay = await page.locator('#tutorial-overlay');

      // Should have ARIA attributes
      await expect(tutorialOverlay).toHaveAttribute('role', 'dialog');
      await expect(tutorialOverlay).toHaveAttribute('aria-modal', 'true');
      await expect(tutorialOverlay).toHaveAttribute('aria-labelledby', 'tutorial-title');
    });
  });

  test.describe('Step 1: Welcome', () => {
    test('should display welcome message', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const welcomeStep = await page.locator('.tutorial-step[data-step="welcome"]');
      await expect(welcomeStep).toBeVisible();

      const title = await welcomeStep.locator('.tutorial-title').textContent();
      expect(title).toContain('Welcome to WOS');
    });

    test('should show Begin Tour button', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const beginButton = await page.locator('#btn-begin-tutorial');
      await expect(beginButton).toBeVisible();
      await expect(beginButton).toHaveText(/Begin Tour/i);
    });

    test('should show Skip Tutorial button', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const skipButton = await page.locator('#btn-skip-tutorial');
      await expect(skipButton).toBeVisible();
      await expect(skipButton).toHaveText(/Skip Tutorial/i);
    });

    test('should advance to Step 2 when Begin Tour clicked', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const beginButton = await page.locator('#btn-begin-tutorial');
      await beginButton.click();
      await page.waitForTimeout(300);

      const terminalStep = await page.locator('.tutorial-step[data-step="terminal"]');
      await expect(terminalStep).toBeVisible();
    });

    test('should close tutorial when Skip clicked', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const skipButton = await page.locator('#btn-skip-tutorial');
      await skipButton.click();
      await page.waitForTimeout(300);

      const tutorialOverlay = await page.locator('#tutorial-overlay');
      await expect(tutorialOverlay).not.toBeVisible();
    });

    test('should mark tutorial as completed in localStorage when skipped', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const skipButton = await page.locator('#btn-skip-tutorial');
      await skipButton.click();
      await page.waitForTimeout(300);

      const completed = await page.evaluate(() => {
        return localStorage.getItem('wos-tutorial-completed');
      });

      expect(completed).toBe('true');
    });
  });

  test.describe('Step 2: Terminal Basics', () => {
    test('should highlight terminal panel', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      // Advance to terminal step
      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);

      const terminal = await page.locator('#terminal');
      const hasHighlight = await terminal.evaluate((el) => {
        return el.classList.contains('tutorial-highlight');
      });

      expect(hasHighlight).toBe(true);
    });

    test('should show glowing border on highlighted element', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);

      const terminal = await page.locator('#terminal');
      const boxShadow = await terminal.evaluate((el) => {
        return window.getComputedStyle(el).boxShadow;
      });

      // Should have a box shadow (glowing effect)
      expect(boxShadow).not.toBe('none');
    });

    test('should prompt user to type ls command', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);

      const terminalStep = await page.locator('.tutorial-step[data-step="terminal"]');
      const instruction = await terminalStep.locator('.tutorial-instruction').textContent();

      expect(instruction).toMatch(/try.*typing.*ls/i);
    });

    test('should advance when user types ls and presses Enter', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);

      // Type ls command
      const terminalInput = await page.locator('#terminal-input');
      await terminalInput.fill('ls');
      await terminalInput.press('Enter');
      await page.waitForTimeout(500);

      // Should advance to next step
      const monitorStep = await page.locator('.tutorial-step[data-step="monitor"]');
      await expect(monitorStep).toBeVisible();
    });

    test('should show Next button if user wants to skip typing', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);

      const nextButton = await page.locator('#btn-tutorial-next');
      await expect(nextButton).toBeVisible();
    });
  });

  test.describe('Step 3: Visual Monitor', () => {
    test('should highlight process view panel', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      // Advance to monitor step
      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click(); // Skip terminal step
      await page.waitForTimeout(300);

      const processView = await page.locator('#panel-system-monitor');
      const hasHighlight = await processView.evaluate((el) => {
        return el.classList.contains('tutorial-highlight');
      });

      expect(hasHighlight).toBe(true);
    });

    test('should prompt user to type ps command', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);

      const monitorStep = await page.locator('.tutorial-step[data-step="monitor"]');
      const instruction = await monitorStep.locator('.tutorial-instruction').textContent();

      expect(instruction).toMatch(/try.*typing.*ps/i);
    });

    test('should explain PID and process states', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);

      const monitorStep = await page.locator('.tutorial-step[data-step="monitor"]');
      const explanation = await monitorStep.textContent();

      expect(explanation).toMatch(/PID/i);
      expect(explanation).toMatch(/state|status/i);
    });
  });

  test.describe('Step 4: Time-Travel Debugging', () => {
    test('should highlight time-travel debugger panel', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      // Advance to debugger step
      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);

      const debuggerPanel = await page.locator('#panel-time-travel-debugger');
      const hasHighlight = await debuggerPanel.evaluate((el) => {
        return el.classList.contains('tutorial-highlight');
      });

      expect(hasHighlight).toBe(true);
    });

    test('should load pre-recorded trace', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);

      // Should have some trace entries
      const eventLog = await page.locator('#event-log-list .event-log-entry');
      const count = await eventLog.count();
      expect(count).toBeGreaterThan(0);
    });

    test('should prompt user to scrub timeline', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);

      const debuggerStep = await page.locator('.tutorial-step[data-step="debugger"]');
      const instruction = await debuggerStep.locator('.tutorial-instruction').textContent();

      expect(instruction).toMatch(/timeline|scrub/i);
    });
  });

  test.describe('Step 5: Test Runner', () => {
    test('should highlight learning objectives panel', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      // Advance to test runner step
      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      for (let i = 0; i < 3; i++) {
        await page.locator('#btn-tutorial-next').click();
        await page.waitForTimeout(300);
      }

      const learningPanel = await page.locator('#panel-learning-objectives');
      const hasHighlight = await learningPanel.evaluate((el) => {
        return el.classList.contains('tutorial-highlight');
      });

      expect(hasHighlight).toBe(true);
    });

    test('should prompt user to click Run Tests', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      for (let i = 0; i < 3; i++) {
        await page.locator('#btn-tutorial-next').click();
        await page.waitForTimeout(300);
      }

      const testStep = await page.locator('.tutorial-step[data-step="tests"]');
      const instruction = await testStep.locator('.tutorial-instruction').textContent();

      expect(instruction).toMatch(/Run Tests/i);
    });

    test('should explain red/green feedback', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      for (let i = 0; i < 3; i++) {
        await page.locator('#btn-tutorial-next').click();
        await page.waitForTimeout(300);
      }

      const testStep = await page.locator('.tutorial-step[data-step="tests"]');
      const explanation = await testStep.textContent();

      expect(explanation).toMatch(/red|green|pass|fail/i);
    });
  });

  test.describe('Step 6: Completion', () => {
    test('should show completion message', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      // Advance to completion step
      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      for (let i = 0; i < 4; i++) {
        await page.locator('#btn-tutorial-next').click();
        await page.waitForTimeout(300);
      }

      const completionStep = await page.locator('.tutorial-step[data-step="completion"]');
      await expect(completionStep).toBeVisible();

      const message = await completionStep.textContent();
      expect(message).toMatch(/Great job|Congrats|Complete/i);
    });

    test('should show Retake Tutorial button', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      for (let i = 0; i < 4; i++) {
        await page.locator('#btn-tutorial-next').click();
        await page.waitForTimeout(300);
      }

      const retakeButton = await page.locator('#btn-retake-tutorial');
      await expect(retakeButton).toBeVisible();
    });

    test('should show Start Coding button', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      for (let i = 0; i < 4; i++) {
        await page.locator('#btn-tutorial-next').click();
        await page.waitForTimeout(300);
      }

      const startButton = await page.locator('#btn-start-coding');
      await expect(startButton).toBeVisible();
      await expect(startButton).toHaveText(/Start Coding/i);
    });

    test('should mark tutorial as completed in localStorage', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      for (let i = 0; i < 4; i++) {
        await page.locator('#btn-tutorial-next').click();
        await page.waitForTimeout(300);
      }

      const completed = await page.evaluate(() => {
        return localStorage.getItem('wos-tutorial-completed');
      });

      expect(completed).toBe('true');
    });

    test('should close tutorial when Start Coding clicked', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      for (let i = 0; i < 4; i++) {
        await page.locator('#btn-tutorial-next').click();
        await page.waitForTimeout(300);
      }

      const startButton = await page.locator('#btn-start-coding');
      await startButton.click();
      await page.waitForTimeout(300);

      const tutorialOverlay = await page.locator('#tutorial-overlay');
      await expect(tutorialOverlay).not.toBeVisible();
    });
  });

  test.describe('Navigation', () => {
    test('should show step progress indicator', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const progressIndicator = await page.locator('.tutorial-progress');
      await expect(progressIndicator).toBeVisible();

      const progressText = await progressIndicator.textContent();
      expect(progressText).toMatch(/Step 1.*6|1 of 6/i);
    });

    test('should update progress as user advances', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);

      const progressIndicator = await page.locator('.tutorial-progress');
      const progressText = await progressIndicator.textContent();
      expect(progressText).toMatch(/Step 2.*6|2 of 6/i);
    });

    test('should allow going back to previous step', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);

      // Now on step 3 (monitor), go back
      const backButton = await page.locator('#btn-tutorial-back');
      await backButton.click();
      await page.waitForTimeout(300);

      // Should be back on step 2 (terminal)
      const terminalStep = await page.locator('.tutorial-step[data-step="terminal"]');
      await expect(terminalStep).toBeVisible();
    });

    test('should not show back button on first step', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const backButton = await page.locator('#btn-tutorial-back');
      await expect(backButton).not.toBeVisible();
    });

    test('should persist current step in localStorage', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);
      await page.locator('#btn-tutorial-next').click();
      await page.waitForTimeout(300);

      const currentStep = await page.evaluate(() => {
        return localStorage.getItem('wos-tutorial-step');
      });

      expect(currentStep).toBe('monitor');
    });

    test('should resume from saved step on reload', async ({ page }) => {
      // Set step to debugger
      await page.evaluate(() => {
        localStorage.setItem('wos-tutorial-step', 'debugger');
      });

      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      // Should show debugger step
      const debuggerStep = await page.locator('.tutorial-step[data-step="debugger"]');
      await expect(debuggerStep).toBeVisible();
    });
  });

  test.describe('Retake Tutorial', () => {
    test('should have retake button in help menu', async ({ page }) => {
      // Mark tutorial as completed
      await page.evaluate(() => {
        localStorage.setItem('wos-tutorial-completed', 'true');
      });

      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

      // Open help menu (assuming there's a help icon)
      const helpButton = await page.locator('#btn-help');
      await helpButton.click();
      await page.waitForTimeout(200);

      const retakeButton = await page.locator('#btn-retake-tutorial-menu');
      await expect(retakeButton).toBeVisible();
    });

    test('should restart tutorial from beginning when retake clicked', async ({ page }) => {
      await page.evaluate(() => {
        localStorage.setItem('wos-tutorial-completed', 'true');
      });

      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

      // Click retake from help menu
      await page.locator('#btn-help').click();
      await page.waitForTimeout(200);
      await page.locator('#btn-retake-tutorial-menu').click();
      await page.waitForTimeout(300);

      // Should show welcome step
      const welcomeStep = await page.locator('.tutorial-step[data-step="welcome"]');
      await expect(welcomeStep).toBeVisible();
    });

    test('should clear tutorial state when retaking', async ({ page }) => {
      await page.evaluate(() => {
        localStorage.setItem('wos-tutorial-completed', 'true');
        localStorage.setItem('wos-tutorial-step', 'completion');
      });

      await page.reload();
      await page.waitForSelector('#status:has-text("Ready")', { timeout: 10000 });

      await page.locator('#btn-help').click();
      await page.waitForTimeout(200);
      await page.locator('#btn-retake-tutorial-menu').click();
      await page.waitForTimeout(300);

      const step = await page.evaluate(() => {
        return localStorage.getItem('wos-tutorial-step');
      });

      expect(step).toBe('welcome');
    });
  });

  test.describe('Accessibility', () => {
    test('should support keyboard navigation with Tab', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      // Tab should focus on Begin Tour button
      await page.keyboard.press('Tab');
      await page.waitForTimeout(100);

      const beginButton = await page.locator('#btn-begin-tutorial');
      const isFocused = await beginButton.evaluate((el) => {
        return el === document.activeElement;
      });

      expect(isFocused).toBe(true);
    });

    test('should support Enter key to activate buttons', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const beginButton = await page.locator('#btn-begin-tutorial');
      await beginButton.focus();
      await page.keyboard.press('Enter');
      await page.waitForTimeout(300);

      // Should advance to terminal step
      const terminalStep = await page.locator('.tutorial-step[data-step="terminal"]');
      await expect(terminalStep).toBeVisible();
    });

    test('should have ARIA live region for step announcements', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const liveRegion = await page.locator('#tutorial-announcer');
      await expect(liveRegion).toBeAttached();
    });

    test('should announce step changes to screen readers', async ({ page }) => {
      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);

      const liveRegion = await page.locator('#tutorial-announcer');
      const announcement = await liveRegion.textContent();

      expect(announcement).toMatch(/Step 2|terminal/i);
    });
  });

  test.describe('Reduced Motion', () => {
    test('should respect prefers-reduced-motion for animations', async ({ page }) => {
      // Set reduced motion preference
      await page.emulateMedia({ reducedMotion: 'reduce' });

      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      const overlay = await page.locator('#tutorial-overlay');
      const transitionDuration = await overlay.evaluate((el) => {
        return window.getComputedStyle(el).transitionDuration;
      });

      // Should have minimal transition duration
      expect(transitionDuration).toMatch(/0s|0.01ms/);
    });

    test('should disable glow animations with reduced motion', async ({ page }) => {
      await page.emulateMedia({ reducedMotion: 'reduce' });

      await page.reload();
      await page.waitForSelector('#tutorial-overlay', { timeout: 10000 });

      await page.locator('#btn-begin-tutorial').click();
      await page.waitForTimeout(300);

      const terminal = await page.locator('#terminal.tutorial-highlight');
      const animationDuration = await terminal.evaluate((el) => {
        return window.getComputedStyle(el).animationDuration;
      });

      // Should have no or minimal animation
      expect(animationDuration).toMatch(/0s|0.01ms/);
    });
  });
});
