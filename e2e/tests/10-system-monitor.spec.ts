import { test, expect } from '@playwright/test';

test.describe('System Monitor Panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('index.html');
    // Clear localStorage to ensure clean state
    await page.evaluate(() => localStorage.clear());
    await page.reload();

    // Wait for WASM to initialize by checking #status element exists with "Ready" text
    // Note: #status might be in a collapsed panel, so we don't check visibility
    await page.waitForFunction(
      () => {
        const status = document.getElementById('status');
        return status && status.textContent === 'Ready';
      },
      { timeout: 10000 }
    );
  });

  test('should display System Monitor panel', async ({ page }) => {
    const monitorPanel = page.locator('.system-monitor-panel');
    await expect(monitorPanel).toBeVisible();

    // Check panel header
    const header = monitorPanel.locator('.file-panel-header h3');
    await expect(header).toHaveText('System Monitor');
  });

  test('should display all four metric cards', async ({ page }) => {
    const cards = page.locator('.monitor-card');

    // Should have exactly 4 cards
    await expect(cards).toHaveCount(4);

    // Verify each card has required elements
    for (let i = 0; i < 4; i++) {
      const card = cards.nth(i);
      await expect(card.locator('.monitor-label')).toBeVisible();
      await expect(card.locator('.monitor-value')).toBeVisible();
      await expect(card.locator('.monitor-subtext')).toBeVisible();
    }
  });

  test('should display CPU usage metric', async ({ page }) => {
    const cpuCard = page.locator('.monitor-card').filter({ hasText: 'CPU Usage' });

    await expect(cpuCard).toBeVisible();
    await expect(cpuCard.locator('.monitor-label')).toHaveText('CPU Usage');
    await expect(cpuCard.locator('#monitor-cpu')).toBeVisible();
    await expect(cpuCard.locator('#monitor-cpu-bar')).toBeVisible();
    await expect(cpuCard.locator('#monitor-cpu-info')).toBeVisible();
  });

  test('should display Memory metric', async ({ page }) => {
    const memCard = page.locator('.monitor-card').filter({ hasText: 'Memory' });

    await expect(memCard).toBeVisible();
    await expect(memCard.locator('.monitor-label')).toHaveText('Memory');
    await expect(memCard.locator('#monitor-memory')).toBeVisible();
    await expect(memCard.locator('#monitor-memory-bar')).toBeVisible();
    await expect(memCard.locator('#monitor-memory-info')).toBeVisible();
  });

  test('should display Processes metric', async ({ page }) => {
    const procCard = page.locator('.monitor-card').filter({ hasText: 'Processes' });

    await expect(procCard).toBeVisible();
    await expect(procCard.locator('.monitor-label')).toHaveText('Processes');
    await expect(procCard.locator('#monitor-processes')).toBeVisible();
    await expect(procCard.locator('#monitor-process-info')).toBeVisible();
  });

  test('should display Syscalls metric', async ({ page }) => {
    const syscallCard = page.locator('.monitor-card').filter({ hasText: 'Syscalls' });

    await expect(syscallCard).toBeVisible();
    await expect(syscallCard.locator('.monitor-label')).toHaveText('Syscalls');
    await expect(syscallCard.locator('#monitor-syscalls')).toBeVisible();
    await expect(syscallCard.locator('#monitor-syscall-info')).toBeVisible();
  });

  test('should update metrics when commands execute', async ({ page }) => {
    const input = page.locator('#terminal-input');
    const cpuValue = page.locator('#monitor-cpu');
    const processesValue = page.locator('#monitor-processes');

    // Get initial process count
    const initialProc = await processesValue.textContent();

    // Execute a command
    await input.fill('ps');
    await input.press('Enter');
    await page.waitForTimeout(500);

    // Process count should update (at least show init process)
    const updatedProc = await processesValue.textContent();
    expect(updatedProc).toBeTruthy();

    // CPU value should be present
    const cpuText = await cpuValue.textContent();
    expect(cpuText).toMatch(/\d+%?/);
  });

  test('should show progress bars for CPU and Memory', async ({ page }) => {
    const cpuBar = page.locator('#monitor-cpu-bar');
    const memBar = page.locator('#monitor-memory-bar');

    // Bars should exist
    await expect(cpuBar).toBeVisible();
    await expect(memBar).toBeVisible();

    // Bars should have width style
    const cpuWidth = await cpuBar.getAttribute('style');
    const memWidth = await memBar.getAttribute('style');

    expect(cpuWidth).toContain('width');
    expect(memWidth).toContain('width');
  });

  test('should have collapse button on System Monitor panel', async ({ page }) => {
    const monitorPanel = page.locator('.system-monitor-panel');
    const collapseBtn = monitorPanel.locator('.btn-collapse');

    await expect(collapseBtn).toBeVisible();
  });
});
