import { test, expect } from '@playwright/test';

test.describe('Mobile Behavior', () => {
  test('should not show mobile bottom bar CTA (removed)', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Mobile bottom bar should not exist
    const mobileBar = page.locator('[data-track="mobile-bottom-cta"]');
    await expect(mobileBar).toHaveCount(0);
  });

  test('should maintain proper mobile viewport functionality without bottom bar', async ({
    page,
  }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Ensure mobile layout works properly without the bottom bar
    const pilotSection = page.locator('#pilot');
    await expect(pilotSection).toBeVisible();

    // Test that form is still accessible on mobile
    const emailInput = page.locator('#email');
    const submitButton = page.locator('#lead-submit');

    // Scroll to pilot section
    await pilotSection.scrollIntoViewIfNeeded();
    await page.waitForTimeout(500);

    // Test form interaction works normally
    await emailInput.fill('mobile-test@example.com');
    await expect(emailInput).toHaveValue('mobile-test@example.com');

    // Submit button should be clickable
    await expect(submitButton).toBeEnabled();
  });

  test('should allow proper scrolling without fixed bottom bar interference', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Test that we can scroll to different sections without interference
    const heroSection = page.locator('section').first();
    const pilotSection = page.locator('#pilot');

    // Scroll to pilot section
    await pilotSection.scrollIntoViewIfNeeded();
    await expect(pilotSection).toBeInViewport({ ratio: 0.1 });

    // Scroll back to top
    await heroSection.scrollIntoViewIfNeeded();
    await expect(heroSection).toBeInViewport({ ratio: 0.1 });
  });
});
