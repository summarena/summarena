import { test, expect } from '@playwright/test';

test.describe('Blog functionality', () => {
  test('should display blog listing page', async ({ page }) => {
    await page.goto('/blog');

    // Check page loads
    await expect(page).toHaveTitle(/Blog.*Summarena/);

    // Check main heading
    await expect(page.locator('h1')).toContainText('Blog');

    // Check header and logo are present
    await expect(page.locator('header')).toBeVisible();
    await expect(page.locator('a[data-track="header-logo-click"]')).toBeVisible();

    // Check navigation exists (even if it's for homepage sections)
    await expect(page.locator('nav')).toBeVisible();
  });

  test('should display individual blog posts', async ({ page }) => {
    // Navigate to blog listing
    await page.goto('/blog');

    // Look for blog post links
    const postLinks = page.locator('article a[href^="/blog/"]');
    const postCount = await postLinks.count();

    if (postCount > 0) {
      // Click on the first blog post
      await postLinks.first().click();

      // Check that we're on a blog post page
      await expect(page.locator('article')).toBeVisible();
      await expect(page.locator('article h1, header h1').first()).toBeVisible();

      // Check that formatted date is present
      await expect(page.locator('time')).toBeVisible();
    }
  });

  test('should have working navigation', async ({ page }) => {
    await page.goto('/blog');

    // Test logo link to home (this actually exists)
    await page.locator('a[data-track="header-logo-click"]').click();
    await expect(page).toHaveURL('/');

    // Navigate back to blog
    await page.goto('/blog');

    // Test CTA button navigation (it links to #pilot relative to current page)
    const ctaButton = page.locator('a[data-track="header-cta-get-started"]');
    if ((await ctaButton.count()) > 0) {
      await ctaButton.click();
      await expect(page).toHaveURL('/blog#pilot');
    }
  });

  test('should have RSS feed available', async ({ page }) => {
    const response = await page.goto('/rss.xml');
    expect(response?.status()).toBe(200);

    const contentType = response?.headers()['content-type'];
    expect(contentType).toContain('xml');

    const content = await response?.text();
    expect(content).toContain('<rss');
    expect(content).toContain('Summarena Blog');
  });

  test('should be responsive on mobile', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/blog');

    // Check that the page is still functional
    await expect(page.locator('h1')).toBeVisible();

    // On mobile, nav is hidden (has class "hidden md:flex")
    const nav = page.locator('nav');
    await expect(nav).toHaveClass(/hidden/);

    // But header should still be visible
    await expect(page.locator('header')).toBeVisible();
    await expect(page.locator('a[data-track="header-logo-click"]')).toBeVisible();
  });

  test('should have all expected blog posts accessible', async ({ page }) => {
    // Go to blog listing
    await page.goto('/blog');

    // Get all blog post links
    const postLinks = page.locator('article a[href^="/blog/"]');
    const postCount = await postLinks.count();

    // Visit each blog post to ensure they're reachable
    for (let i = 0; i < postCount; i++) {
      const link = postLinks.nth(i);
      const href = await link.getAttribute('href');
      if (href) {
        const response = await page.goto(href);
        expect(response?.status()).toBe(200);

        // Check basic blog post structure
        await expect(page.locator('article')).toBeVisible();
        await expect(page.locator('article h1, header h1').first()).toBeVisible();
        await expect(page.locator('time')).toBeVisible();

        // Go back to blog listing for next iteration
        await page.goto('/blog');
      }
    }
  });
});
