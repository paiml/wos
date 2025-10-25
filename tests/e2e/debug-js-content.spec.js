// Debug test to check what JavaScript is actually loaded
const { test, expect } = require('@playwright/test');

test('debug javascript content', async ({ page, context }) => {
  // Clear cache
  await context.clearCookies();

  // Intercept app.js request to see what's being loaded
  const appJsContent = await new Promise((resolve) => {
    page.on('response', async (response) => {
      if (response.url().includes('app.js')) {
        const text = await response.text();
        resolve(text);
      }
    });
    page.goto('http://127.0.0.1:8000', { waitUntil: 'networkidle' });
  });

  // Check if it contains our fix
  const hasNewCode = appJsContent.includes('Use simple \'ls\' command (ls -la is broken in WOS)');
  const hasOldCode = appJsContent.includes('ls -la /');

  console.log('=== APP.JS ANALYSIS ===');
  console.log(`Has new code comment: ${hasNewCode}`);
  console.log(`Has old "ls -la /" code: ${hasOldCode}`);

  // Find the executeCommand call in updateFilesystemList
  const match = appJsContent.match(/updateFilesystemList[\s\S]{0,500}executeCommand\('([^']+)'\)/);
  if (match) {
    console.log(`Actual ls command used: "${match[1]}"`);
  } else {
    console.log('Could not find executeCommand in updateFilesystemList');
  }
});
