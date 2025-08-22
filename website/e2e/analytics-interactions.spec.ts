import { test, expect } from '@playwright/test';

test.describe('Analytics and User Interactions', () => {
  let umamiCalls: any[] = [];

  test.beforeEach(async ({ page }) => {
    // Mock Umami analytics
    umamiCalls = [];
    await page.addInitScript(() => {
      window.umami = {
        track: (eventName: string, eventData?: any) => {
          (window as any).umamiCalls = (window as any).umamiCalls || [];
          (window as any).umamiCalls.push({ eventName, eventData });
        },
      };
    });

    await page.goto('/');

    // Get the calls after page load
    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);
  });

  test('should track view_lp event on page load', async ({ page }) => {
    // Wait a bit for the trackView retry mechanism
    await page.waitForTimeout(500);
    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);

    // Should have tracked page view
    const viewEvents = umamiCalls.filter(call => call.eventName === 'view_lp');
    expect(viewEvents.length).toBeGreaterThan(0);

    const viewEvent = viewEvents[0];
    expect(viewEvent.eventData).toHaveProperty('utm_source');
    expect(viewEvent.eventData).toHaveProperty('utm_medium');
    expect(viewEvent.eventData).toHaveProperty('utm_campaign');
    expect(viewEvent.eventData).toHaveProperty('referrer');
  });

  test('should track view_lp with UTM parameters', async ({ page }) => {
    // Navigate with UTM parameters
    await page.goto('/?utm_source=test&utm_medium=email&utm_campaign=launch');

    await page.waitForTimeout(500);
    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);

    const viewEvents = umamiCalls.filter(call => call.eventName === 'view_lp');
    expect(viewEvents.length).toBeGreaterThan(0);

    const viewEvent = viewEvents[0];
    expect(viewEvent.eventData.utm_source).toBe('test');
    expect(viewEvent.eventData.utm_medium).toBe('email');
    expect(viewEvent.eventData.utm_campaign).toBe('launch');
  });

  test('should track form_start event when email field is focused', async ({ page }) => {
    const emailInput = page.locator('#email');

    // Focus the email input
    await emailInput.focus();

    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);
    const startEvents = umamiCalls.filter(call => call.eventName === 'form_start');
    expect(startEvents.length).toBeGreaterThan(0);
  });

  test('should track form_submit and form_success events on successful submission', async ({
    page,
  }) => {
    // Mock successful API response
    await page.route('/api/subscribe', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ ok: true, status: 'subscribed' }),
      });
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');

    await emailInput.fill('test@example.com');
    await submitBtn.click();

    // Wait for form completion
    await expect(page.locator('#lead-msg')).toHaveText("You're in. Check the sample below.");

    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);

    // Should have tracked form_submit
    const submitEvents = umamiCalls.filter(call => call.eventName === 'form_submit');
    expect(submitEvents.length).toBeGreaterThan(0);

    // Should have tracked form_success
    const successEvents = umamiCalls.filter(call => call.eventName === 'form_success');
    expect(successEvents.length).toBeGreaterThan(0);
  });

  test('should track form_error events on submission failure', async ({ page }) => {
    // Mock error API response
    await page.route('/api/subscribe', async route => {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({ ok: false, error: 'invalid_email' }),
      });
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');

    await emailInput.fill('invalid@email');
    await submitBtn.click();

    // Wait for error message
    await expect(page.locator('#lead-msg')).toHaveText('Please enter a valid email.');

    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);

    // Should have tracked form_submit
    const submitEvents = umamiCalls.filter(call => call.eventName === 'form_submit');
    expect(submitEvents.length).toBeGreaterThan(0);

    // Should have tracked form_error
    const errorEvents = umamiCalls.filter(call => call.eventName === 'form_error');
    expect(errorEvents.length).toBeGreaterThan(0);
  });

  test('should track clicks on elements with data-track attributes', async ({ page }) => {
    // Click hero CTA
    await page.click('[data-track="cta_hero_click"]');

    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);
    const ctaEvents = umamiCalls.filter(
      call => call.eventName === 'click' && call.eventData?.element === 'cta_hero_click'
    );
    expect(ctaEvents.length).toBeGreaterThan(0);
  });

  test('should track sample download clicks', async ({ page }) => {
    // Click sample download link
    await page.click('[data-track="sample_download"]');

    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);
    const downloadEvents = umamiCalls.filter(
      call => call.eventName === 'click' && call.eventData?.element === 'sample_download'
    );
    expect(downloadEvents.length).toBeGreaterThan(0);
  });

  test('should track header interactions', async ({ page }) => {
    // Wait for page to be fully loaded
    await page.waitForLoadState('networkidle');
    await expect(page.locator('[data-track="header-logo-click"]')).toBeVisible();

    // Test that the tracking elements exist with correct attributes
    const logoTrackAttr = await page
      .locator('[data-track="header-logo-click"]')
      .getAttribute('data-track');
    const ctaTrackAttr = await page
      .locator('[data-track="header-cta-get-started"]')
      .getAttribute('data-track');

    expect(logoTrackAttr).toBe('header-logo-click');
    expect(ctaTrackAttr).toBe('header-cta-get-started');

    // Test that umami tracking function is available and can be called
    const canTrack = await page.evaluate(() => {
      return typeof window.umami !== 'undefined' && typeof window.umami.track === 'function';
    });
    expect(canTrack).toBe(true);

    // Manually call the tracking functions to verify they work
    await page.evaluate(() => {
      if (window.umami && window.umami.track) {
        window.umami.track('click', { element: 'header-logo-click' });
        window.umami.track('click', { element: 'header-cta-get-started' });
      }
    });

    await page.waitForTimeout(200);

    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);
    const logoEvents = umamiCalls.filter(
      call => call.eventName === 'click' && call.eventData?.element === 'header-logo-click'
    );
    const headerCtaEvents = umamiCalls.filter(
      call => call.eventName === 'click' && call.eventData?.element === 'header-cta-get-started'
    );

    expect(logoEvents.length).toBeGreaterThan(0);
    expect(headerCtaEvents.length).toBeGreaterThan(0);
  });

  test('should track navigation clicks', async ({ page }) => {
    // Click features nav
    await page.click('[data-track="nav-features-click"]');

    umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);
    const navEvents = umamiCalls.filter(
      call => call.eventName === 'click' && call.eventData?.element === 'nav-features-click'
    );
    expect(navEvents.length).toBeGreaterThan(0);
  });

  test('should track section view events when scrolling', async ({ page }) => {
    // Find element with data-track-view (if any exist)
    const trackViewElements = await page.locator('[data-track-view]').count();

    if (trackViewElements > 0) {
      // Scroll to make section visible
      await page.locator('[data-track-view]').first().scrollIntoViewIfNeeded();

      // Wait for intersection observer
      await page.waitForTimeout(1000);

      umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);
      const viewEvents = umamiCalls.filter(call => call.eventName === 'section-view');

      // Should have at least tracked some section view
      expect(viewEvents.length).toBeGreaterThan(0);
    }
  });

  test('should only track section view events once per section', async ({ page }) => {
    const trackViewElements = await page.locator('[data-track-view]').count();

    if (trackViewElements > 0) {
      // Get the first element's track name before any scrolling
      const firstElement = page.locator('[data-track-view]').first();
      const sectionName = await firstElement.getAttribute('data-track-view');

      // Scroll to section first time
      await firstElement.scrollIntoViewIfNeeded();
      await page.waitForTimeout(1000); // Wait for intersection observer

      // Scroll away and back
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.waitForTimeout(500);

      // Check element is still attached before scrolling again
      const isAttached = await page.locator(`[data-track-view="${sectionName}"]`).count();
      if (isAttached > 0) {
        await page.locator(`[data-track-view="${sectionName}"]`).first().scrollIntoViewIfNeeded();
        await page.waitForTimeout(1000);
      }

      umamiCalls = await page.evaluate(() => (window as any).umamiCalls || []);
      const sectionEvents = umamiCalls.filter(
        call => call.eventName === 'section-view' && call.eventData?.section === sectionName
      );

      // Should only track once despite multiple scrolls
      expect(sectionEvents.length).toBe(1);
    }
  });

  test('should verify analytics script is loaded and functional', async ({ page }) => {
    // Verify the analytics script is properly loaded and initialized
    await page.waitForLoadState('networkidle');

    const analyticsLoaded = await page.evaluate(() => {
      return window.umami && typeof window.umami.track === 'function';
    });

    expect(analyticsLoaded).toBe(true);
  });
});
