import { test, expect } from '@playwright/test';

/**
 * WebOS Showcase Demo - 30-second comprehensive demonstration
 *
 * This test showcases:
 * 1. Vim mode editing
 * 2. Bash scripting with environment variables
 * 3. File system operations
 * 4. Process management
 *
 * Video output: test-results/wos-showcase-demo/video.webm
 */

test.describe('WebOS Showcase Demo', () => {
    test.beforeEach(async ({ page }) => {
        // Set viewport for demo recording
        await page.setViewportSize({ width: 1920, height: 1080 });
    });

    test('30-second comprehensive WebOS demo', async ({ page }) => {
        test.setTimeout(60000); // 60 second timeout for the demo
        // Navigate to WebOS
        await page.goto('http://localhost:8000/wos/');

        // Wait for WebOS to fully load
        await page.waitForSelector('#terminal', { timeout: 10000 });
        await page.waitForTimeout(2000); // Let UI settle

        // Dismiss tutorial overlay if present
        const skipButton = page.locator('button:has-text("Skip Tutorial")');
        if (await skipButton.isVisible({ timeout: 2000 }).catch(() => false)) {
            await skipButton.click();
            await page.waitForTimeout(500);
        }

        // Get terminal element
        const terminal = page.locator('#terminal');

        // ============================================
        // Part 1: Set up environment variables (3s)
        // ============================================
        console.log('📝 Setting environment variables...');

        await terminal.click();
        await page.keyboard.type('export NAME="WebOS Demo"');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500);

        await page.keyboard.type('export VERSION="1.0"');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500);

        // ============================================
        // Part 2: Create bash script using vim (8s)
        // ============================================
        console.log('✍️  Creating bash script in vim...');

        await page.keyboard.type('vim demo.sh');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        // Enter insert mode
        await page.keyboard.press('i');
        await page.waitForTimeout(300);

        // Write bash script with environment variables
        const script = `#!/bin/bash
echo "=== $NAME ==="
echo "Version: $VERSION"
echo "User: $USER"
echo "Working Dir: $PWD"
ls -la`;

        await page.keyboard.type(script);
        await page.waitForTimeout(1000);

        // Exit insert mode and save
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
        await page.keyboard.type(':wq');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        // ============================================
        // Part 3: Make script executable and run (5s)
        // ============================================
        console.log('🚀 Executing bash script...');

        await page.keyboard.type('chmod +x demo.sh');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500);

        await page.keyboard.type('./demo.sh');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(2000);

        // ============================================
        // Part 4: Show file system operations (4s)
        // ============================================
        console.log('📁 File system operations...');

        await page.keyboard.type('cat demo.sh');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(2000);

        // ============================================
        // Part 5: Edit file again with vim (3s)
        // ============================================
        console.log('✏️  Editing with vim again...');

        await page.keyboard.type('vim demo.sh');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        // Add a comment using vim
        await page.keyboard.press('i');
        await page.waitForTimeout(300);
        await page.keyboard.type('# WebOS Demo Script');
        await page.keyboard.press('Enter');
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
        await page.keyboard.type(':wq');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        // ============================================
        // Part 6: Process management (2s)
        // ============================================
        console.log('⚙️  Process management...');

        await page.keyboard.type('ps');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1500);

        // ============================================
        // Final: Show env vars and complete (2s)
        // ============================================
        await page.keyboard.type('env | grep -E "NAME|VERSION"');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        await page.keyboard.type('echo "🎉 Demo Complete!"');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        console.log('✅ Demo complete! Video saved to test-results/');
    });
});
