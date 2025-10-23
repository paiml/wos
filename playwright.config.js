import { defineConfig } from '@playwright/test';

export default defineConfig({
    testDir: './tests/e2e',
    fullyParallel: false,
    forbidOnly: !!process.env.CI,
    retries: 0,
    workers: 1,
    reporter: 'list',

    use: {
        baseURL: 'http://localhost:8000',
        trace: 'retain-on-failure',
        video: 'retain-on-failure', // Record video on failure
        screenshot: 'only-on-failure',
    },

    // For demo tests, always record video
    projects: [
        {
            name: 'demo',
            testMatch: '**/wos-showcase-demo.spec.js',
            use: {
                video: 'on', // Always record video for demo
            },
        },
        {
            name: 'default',
            testIgnore: '**/wos-showcase-demo.spec.js',
        },
    ],
});
