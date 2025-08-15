import { test, expect } from '@playwright/test';

test.describe('Homepage', () => {
  test('should load homepage successfully', async ({ page }) => {
    await page.goto('/');

    // Check page loads
    await expect(page).toHaveTitle(/SaaS Project/);

    // Check main content is visible
    await expect(page.locator('h1')).toBeVisible();

    // Check navigation links work
    const blogLink = page.locator('nav a[href="/blog"]');
    if ((await blogLink.count()) > 0) {
      await expect(blogLink).toBeVisible();
    }

    const docsLink = page.locator('nav a[href^="/docs"]');
    if ((await docsLink.count()) > 0) {
      await expect(docsLink).toBeVisible();
    }
  });

  test('should have working navigation to blog', async ({ page }) => {
    await page.goto('/');

    const blogLink = page.locator('nav a[href="/blog"]').first();
    if ((await blogLink.count()) > 0) {
      await blogLink.click();
      await expect(page).toHaveURL('/blog');
      await expect(page.locator('h1')).toContainText('Blog');
    }
  });

  test('should have working navigation to docs', async ({ page }) => {
    await page.goto('/');

    const docsLink = page.locator('nav a[href^="/docs"]').first();
    if ((await docsLink.count()) > 0) {
      await docsLink.click();
      await expect(page).toHaveURL(/\/docs/);
    }
  });
});
