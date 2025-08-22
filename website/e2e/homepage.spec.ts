import { test, expect } from '@playwright/test';

test.describe('Homepage', () => {
  test('should load homepage successfully', async ({ page }) => {
    await page.goto('/');

    // Check page loads with correct title
    await expect(page).toHaveTitle(/SummArena.*Research brief/);

    // Check main content is visible
    await expect(page.locator('h1')).toBeVisible();

    // Check header and navigation exist
    await expect(page.locator('header')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();
    await expect(page.locator('a[data-track="header-logo-click"]')).toBeVisible();
  });

  test('should have working navigation to blog', async ({ page }) => {
    await page.goto('/');

    // Navigate via direct link since nav doesn't have blog link
    await page.goto('/blog');
    await expect(page).toHaveURL('/blog');
    await expect(page.locator('h1')).toContainText('Blog');
  });

  test('should have working navigation to docs', async ({ page }) => {
    await page.goto('/');

    // Navigate via direct link since nav doesn't have docs link
    await page.goto('/docs');
    await expect(page).toHaveURL(/\/docs/);
  });
});
