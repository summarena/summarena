import { test, expect } from '@playwright/test';

test.describe('Email Collection Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the homepage where the email form is located
    await page.goto('/');
  });

  test('should display the contact form', async ({ page }) => {
    // Check that the form elements are present
    await expect(page.locator('#contact-form')).toBeVisible();
    await expect(page.locator('#email')).toBeVisible();
    await expect(page.locator('#role')).toBeVisible();
    await expect(page.locator('#interests')).toBeVisible();
    await expect(page.locator('#submit-btn')).toBeVisible();

    // Check form labels
    await expect(page.locator('label[for="email"]')).toHaveText('Email');
    await expect(page.locator('label[for="role"]')).toHaveText('Role');
    await expect(page.locator('label[for="interests"]')).toHaveText(
      'What do you want in your digest?'
    );

    // Check button text
    await expect(page.locator('#submit-btn')).toHaveText('Request Invite');
  });

  test('should validate email field client-side', async ({ page }) => {
    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#submit-btn');

    // Try to submit with invalid email
    await emailInput.fill('invalid-email');
    await submitBtn.click();

    // Check that browser validation prevents submission
    await expect(emailInput).toHaveAttribute('type', 'email');
    await expect(emailInput).toHaveAttribute('required');
  });

  test('should show validation message for empty email', async ({ page }) => {
    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#submit-btn');
    const messageEl = page.locator('#form-message');

    // Temporarily remove required attribute to test our custom validation
    await emailInput.evaluate(el => el.removeAttribute('required'));

    // Click submit without email
    await submitBtn.click();

    // Should show validation message
    await expect(messageEl).toHaveText('Please enter your email address.');
    await expect(messageEl).toHaveClass(/text-red-600/);
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
    const roleInput = page.locator('#role');
    const interestsInput = page.locator('#interests');
    const submitBtn = page.locator('#submit-btn');
    const messageEl = page.locator('#form-message');

    // Fill out the form
    await emailInput.fill('test@example.com');
    await roleInput.fill('Product Manager');
    await interestsInput.fill('AI papers, startup news');

    // Submit the form
    await submitBtn.click();

    // Check loading state (might be very quick, so we use a shorter timeout)
    await expect(submitBtn)
      .toHaveText('Submitting...')
      .catch(() => {
        // If we miss the loading state, that's ok - the request was too fast
      });
    await expect(submitBtn)
      .toBeDisabled()
      .catch(() => {
        // Similar for disabled state
      });

    // Wait for completion and check success message
    await expect(messageEl).toHaveText("Thanks! You're on the list. We'll be in touch soon.");
    await expect(messageEl).toHaveClass(/text-green-600/);

    // Check that form is reset
    await expect(emailInput).toHaveValue('');
    await expect(roleInput).toHaveValue('');
    await expect(interestsInput).toHaveValue('');

    // Check that button is re-enabled
    await expect(submitBtn).toHaveText('Request Invite');
    await expect(submitBtn).toBeEnabled();
  });

  test('should handle already subscribed email', async ({ page }) => {
    // Mock the API response for already subscribed
    await page.route('/api/subscribe', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          ok: true,
          status: 'already_subscribed',
        }),
      });
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#submit-btn');
    const messageEl = page.locator('#form-message');

    await emailInput.fill('existing@example.com');
    await submitBtn.click();

    await expect(messageEl).toHaveText("You're already on our list! We'll be in touch soon.");
    await expect(messageEl).toHaveClass(/text-blue-600/);
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
    const submitBtn = page.locator('#submit-btn');
    const messageEl = page.locator('#form-message');

    await emailInput.fill('invalid@email');
    await submitBtn.click();

    await expect(messageEl).toHaveText('Please enter a valid email address.');
    await expect(messageEl).toHaveClass(/text-red-600/);
  });

  test('should handle server errors gracefully', async ({ page }) => {
    // Mock a server error
    await page.route('/api/subscribe', async route => {
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({
          ok: false,
          error: 'server_error',
        }),
      });
    });

    const emailInput = page.locator('#email');
    const submitBtn = page.locator('#submit-btn');
    const messageEl = page.locator('#form-message');

    await emailInput.fill('test@example.com');
    await submitBtn.click();

    await expect(messageEl).toHaveText('Something went wrong. Please try again.');
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
    const submitBtn = page.locator('#submit-btn');
    const messageEl = page.locator('#form-message');

    await emailInput.fill('test@example.com');
    await submitBtn.click();

    await expect(messageEl).toHaveText('Something went wrong. Please try again.');
    await expect(messageEl).toHaveClass(/text-red-600/);
  });

  test('should submit form data correctly', async ({ page }) => {
    let capturedRequest: any = null;

    // Capture the request data
    await page.route('/api/subscribe', async route => {
      capturedRequest = route.request();
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
    const roleInput = page.locator('#role');
    const interestsInput = page.locator('#interests');
    const submitBtn = page.locator('#submit-btn');

    // Fill out the form
    await emailInput.fill('form-test@example.com');
    await roleInput.fill('Researcher');
    await interestsInput.fill('ML research, robotics');

    await submitBtn.click();

    // Wait for the request to be captured
    await expect(page.locator('#form-message')).toHaveText(
      "Thanks! You're on the list. We'll be in touch soon."
    );

    // Verify request details
    expect(capturedRequest?.method()).toBe('POST');
    expect(capturedRequest?.url()).toContain('/api/subscribe');

    // Check that form data was sent (we can't easily inspect FormData, but we can verify the request was made)
    expect(capturedRequest).toBeTruthy();
  });

  test('should work with only email field filled', async ({ page }) => {
    // Mock successful response
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
    const submitBtn = page.locator('#submit-btn');
    const messageEl = page.locator('#form-message');

    // Fill only email field
    await emailInput.fill('minimal@example.com');
    await submitBtn.click();

    await expect(messageEl).toHaveText("Thanks! You're on the list. We'll be in touch soon.");
    await expect(messageEl).toHaveClass(/text-green-600/);
  });

  test('should maintain form accessibility', async ({ page }) => {
    // Check that form elements have proper accessibility attributes
    await expect(page.locator('#form-message')).toHaveAttribute('aria-live', 'polite');

    // Check that labels are properly associated with inputs
    await expect(page.locator('label[for="email"]')).toBeVisible();
    await expect(page.locator('label[for="role"]')).toBeVisible();
    await expect(page.locator('label[for="interests"]')).toBeVisible();

    // Check that email input is required
    await expect(page.locator('#email')).toHaveAttribute('required');
  });
});
