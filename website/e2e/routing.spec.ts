import { test, expect } from '@playwright/test';

test.describe('Site routing and navigation', () => {
  const routes = [
    { path: '/', title: /SaaS Project/ },
    { path: '/blog', title: /Blog.*Summarena/ },
    { path: '/docs/guides/getting-started', title: /Getting Started/ },
    { path: '/rss.xml', contentType: 'xml' },
  ];

  test('should have all main routes accessible', async ({ page }) => {
    for (const route of routes) {
      const response = await page.goto(route.path);
      expect(response?.status()).toBe(200);

      if (route.contentType === 'xml') {
        const contentType = response?.headers()['content-type'];
        expect(contentType).toContain('xml');
      } else if (route.title) {
        await expect(page).toHaveTitle(route.title);
      }
    }
  });

  test('should have working navigation between pages', async ({ page }) => {
    // Start from homepage
    await page.goto('/');

    // Navigate to blog
    await page.locator('nav a[href="/blog"]').first().click();
    await expect(page).toHaveURL('/blog');
    await expect(page.locator('h1')).toContainText('Blog');

    // Navigate back to home from blog
    await page.locator('nav a[href="/"]').first().click();
    await expect(page).toHaveURL('/');

    // Navigate to docs if available
    const docsLink = page.locator('nav a[href^="/docs"]').first();
    if ((await docsLink.count()) > 0) {
      await docsLink.click();
      await expect(page).toHaveURL(/\/docs/);
    }
  });

  test('should handle 404 for non-existent pages', async ({ page }) => {
    const response = await page.goto('/non-existent-page');
    expect(response?.status()).toBe(404);
  });

  test('should have consistent navigation across all pages', async ({ page }) => {
    const pagesToCheck = ['/', '/blog'];

    for (const pagePath of pagesToCheck) {
      await page.goto(pagePath);

      // Check that navigation elements exist
      const nav = page.locator('nav, header nav');
      await expect(nav).toBeVisible();

      // Check for home link
      const homeLink = page.locator('nav a[href="/"]');
      await expect(homeLink.first()).toBeVisible();

      // Check for blog link
      const blogLink = page.locator('nav a[href="/blog"]');
      await expect(blogLink.first()).toBeVisible();
    }
  });

  test('should have proper cross-linking between blog posts and pages', async ({ page }) => {
    // Go to a blog post
    await page.goto('/blog');
    const firstPostLink = page.locator('article a[href^="/blog/"]').first();

    if ((await firstPostLink.count()) > 0) {
      await firstPostLink.click();

      // Check that we can navigate back to blog listing
      const blogLink = page.locator('nav a[href="/blog"]');
      if ((await blogLink.count()) > 0) {
        await blogLink.click();
        await expect(page).toHaveURL('/blog');
      }
    }
  });

  test('should handle routing with trailing slashes correctly', async ({ page }) => {
    const routesToTest = [{ original: '/blog', withSlash: '/blog/' }];

    for (const route of routesToTest) {
      // Test original
      const response1 = await page.goto(route.original);
      expect(response1?.status()).toBe(200);

      // Test with trailing slash
      const response2 = await page.goto(route.withSlash);
      expect(response2?.status()).toBe(200);
    }
  });
});
