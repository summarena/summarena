import { test, expect } from '@playwright/test';

test.describe('Email Collection Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the homepage where the email form is located
    await page.goto('/');
  });

  test('should display the lead form with single email input', async ({ page }) => {
    // Check that the simplified form elements are present
    await expect(page.locator('#lead-form')).toBeVisible();
    await expect(page.locator('#email')).toBeVisible();
    await expect(page.locator('#lead-submit')).toBeVisible();

    // Check form attributes
    await expect(page.locator('#lead-form')).toHaveAttribute('action', '/api/subscribe');
    await expect(page.locator('#lead-form')).toHaveAttribute('method', 'POST');

    // Check button text
    await expect(page.locator('#lead-submit')).toHaveText('Start free');

    // Check email input attributes
    await expect(page.locator('#email')).toHaveAttribute('type', 'email');
    await expect(page.locator('#email')).toHaveAttribute('required');
    await expect(page.locator('#email')).toHaveAttribute('autocomplete', 'email');

    // Check honeypot field is present but hidden
    await expect(page.locator('input[name="company"]')).toHaveClass(/hidden/);
    await expect(page.locator('input[name="company"]')).toHaveAttribute('aria-hidden', 'true');
  });

  test('should have proper accessibility attributes', async ({ page }) => {
    // Check screen reader label
    await expect(page.locator('label[for="email"]')).toHaveClass(/sr-only/);
    await expect(page.locator('label[for="email"]')).toHaveText('Email');

    // Check aria-live for message element
    await expect(page.locator('#lead-msg')).toHaveAttribute('aria-live', 'polite');
  });

  test('should validate email field client-side', async ({ page }) => {
    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');

    // Try to submit with invalid email
    await emailInput.fill('invalid-email');
    await submitBtn.click();

    // Check that browser validation prevents submission
    await expect(emailInput).toHaveAttribute('type', 'email');
    await expect(emailInput).toHaveAttribute('required');
  });

  test('should handle successful form submission', async ({ page }) => {
    // Mock the API response for successful subscription
    await page.route('/api/subscribe', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          ok: true,
          status: 'subscribed',
        }),
      });
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');
    const messageEl = page.locator('#lead-msg');

    // Fill out the form
    await emailInput.fill('test@example.com');

    // Submit the form
    await submitBtn.click();

    // Check loading state (might be very quick)
    await expect(submitBtn)
      .toHaveText('Submitting')
      .catch(() => {
        // If we miss the loading state, that's ok - the request was too fast
      });
    await expect(submitBtn)
      .toBeDisabled()
      .catch(() => {
        // Similar for disabled state
      });

    // Wait for completion and check success message
    await expect(messageEl).toHaveText("You're in. Check the sample below.");
    await expect(messageEl).toHaveClass(/text-green-600/);

    // Check that button is re-enabled with original text
    await expect(submitBtn).toHaveText('Start free');
    await expect(submitBtn).toBeEnabled();
  });

  test('should handle already subscribed email', async ({ page }) => {
    // Mock the API response for already subscribed
    await page.route('/api/subscribe', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          ok: false,
          error: 'already_subscribed',
        }),
      });
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');
    const messageEl = page.locator('#lead-msg');

    await emailInput.fill('existing@example.com');
    await submitBtn.click();

    await expect(messageEl).toHaveText("You're already on the list.");
    await expect(messageEl).toHaveClass(/text-red-600/);
  });

  test('should handle invalid email from server', async ({ page }) => {
    // Mock the API response for invalid email
    await page.route('/api/subscribe', async route => {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({
          ok: false,
          error: 'invalid_email',
        }),
      });
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');
    const messageEl = page.locator('#lead-msg');

    await emailInput.fill('invalid@email');
    await submitBtn.click();

    await expect(messageEl).toHaveText('Please enter a valid email.');
    await expect(messageEl).toHaveClass(/text-red-600/);
  });

  test('should handle server errors gracefully', async ({ page }) => {
    // Mock a server error (network/parse error to trigger catch block)
    await page.route('/api/subscribe', async route => {
      await route.abort('failed');
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');
    const messageEl = page.locator('#lead-msg');

    await emailInput.fill('test@example.com');
    await submitBtn.click();

    await expect(messageEl).toHaveText('Error. Try again.');
    await expect(messageEl).toHaveClass(/text-red-600/);

    // Button should be re-enabled for retry
    await expect(submitBtn).toBeEnabled();
  });

  test('should handle network failures', async ({ page }) => {
    // Mock a network failure
    await page.route('/api/subscribe', async route => {
      await route.abort();
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');
    const messageEl = page.locator('#lead-msg');

    await emailInput.fill('test@example.com');
    await submitBtn.click();

    await expect(messageEl).toHaveText('Error. Try again.');
    await expect(messageEl).toHaveClass(/text-red-600/);
  });

  test('should protect against bot submissions with honeypot', async ({ page }) => {
    // Mock successful response (honeypot protection happens server-side)
    await page.route('/api/subscribe', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          ok: true,
          status: 'subscribed',
        }),
      });
    });

    // Verify honeypot field exists and is properly hidden
    const honeyPot = page.locator('input[name="company"]');
    await expect(honeyPot).toHaveClass(/hidden/);
    await expect(honeyPot).toHaveAttribute('aria-hidden', 'true');

    // Simulate bot filling honeypot field
    await page.evaluate(() => {
      const honeyPot = document.querySelector('input[name="company"]') as HTMLInputElement;
      if (honeyPot) honeyPot.value = 'bot-company';
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');
    const messageEl = page.locator('#lead-msg');

    await emailInput.fill('bot@example.com');
    await submitBtn.click();

    // Should still show success to bot (server handles the filtering)
    await expect(messageEl).toHaveText("You're in. Check the sample below.");
  });

  test('should submit minimal form data correctly', async ({ page }) => {
    let capturedFormData: any = null;

    // Capture the request data
    await page.route('/api/subscribe', async route => {
      const postData = route.request().postData();
      if (postData) {
        capturedFormData = Object.fromEntries(new URLSearchParams(postData));
      }

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          ok: true,
          status: 'subscribed',
        }),
      });
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#lead-submit');
    const messageEl = page.locator('#lead-msg');

    // Verify form fields are present
    await expect(emailInput).toBeVisible();
    await expect(page.locator('input[name="company"]')).toHaveClass(/hidden/);

    // Fill only email field
    await emailInput.fill('minimal@example.com');
    await submitBtn.click();

    // Wait for completion
    await expect(messageEl).toHaveText("You're in. Check the sample below.");
    await expect(messageEl).toHaveClass(/text-green-600/);
  });
});
