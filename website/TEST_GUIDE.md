# Testing Guide for SummArena Landing Page

This guide explains how to run the comprehensive test suite for the refactored landing page.

## Test Suite Overview

Our test suite includes:

- **Playwright E2E Tests**: End-to-end browser testing of form behavior, analytics, and mobile interactions
- **Wrangler Integration Tests**: Tests the actual Cloudflare workerd runtime environment
- **Unit Tests**: Component and function testing (if applicable)

## Prerequisites

1. Build the project:

   ```bash
   pnpm build
   ```

2. Ensure Wrangler is available (for integration tests):

   ```bash
   npx wrangler --version
   ```

3. Install Playwright browsers (if not already installed):
   ```bash
   pnpm exec playwright install
   ```

## Running Tests

### All Tests

Run the complete test suite:

```bash
pnpm test:all
```

### Playwright E2E Tests

Run end-to-end browser tests:

```bash
pnpm test:e2e
```

Run with Playwright UI (interactive mode):

```bash
pnpm test:e2e:ui
```

Run in headed mode (see browser):

```bash
pnpm exec playwright test --headed
```

### Wrangler Integration Tests

Run tests in the actual Cloudflare workerd environment:

```bash
pnpm test:integration
```

### Individual Test Files

Run specific test files:

```bash
# Form behavior tests
pnpm exec playwright test e2e/email-collection.spec.ts

# Analytics tracking tests
pnpm exec playwright test e2e/analytics-interactions.spec.ts

# Mobile behavior tests
pnpm exec playwright test e2e/mobile-behavior.spec.ts

# Wrangler integration tests
pnpm exec vitest tests/integration/wrangler-pages.test.ts
```

## Test Coverage

### Form Functionality Tests (`e2e/email-collection.spec.ts`)

- ✅ Single email input validation
- ✅ Honeypot spam protection
- ✅ Form submission and API responses
- ✅ Success and error message display
- ✅ Already subscribed handling

### Analytics Tests (`e2e/analytics-interactions.spec.ts`)

- ✅ Page view tracking (`view_lp`)
- ✅ UTM parameter capture
- ✅ Form interaction events (`form_start`, `form_submit`, `form_success`, `form_error`)
- ✅ Click tracking for CTAs and links
- ✅ Section view tracking
- ✅ Header and navigation interactions

### Mobile Behavior Tests (`e2e/mobile-behavior.spec.ts`)

- ✅ Mobile bottom bar CTA visibility
- ✅ Intersection observer behavior (hides when pilot section visible)
- ✅ Mobile viewport responsive design
- ✅ Touch interaction compatibility

### Wrangler Integration Tests (`tests/integration/wrangler-pages.test.ts`)

- ✅ Actual workerd runtime environment
- ✅ API endpoint functionality
- ✅ Honeypot protection server-side
- ✅ Email normalization and deduplication
- ✅ CORS handling
- ✅ Metadata and structured data

## Test Environment Configuration

### Playwright Configuration

Located in `playwright.config.ts`:

- Tests run in Chromium, Firefox, and WebKit
- Mobile and desktop viewports tested
- Screenshot capture on failure
- Video recording for debugging

### Wrangler Test Configuration

Integration tests use:

- Local KV store binding (`EMAILS`)
- Mock webhook URL for Google Sheets
- Port 8788 for test server
- Full workerd runtime simulation

## Expected Test Results

All tests should pass with the new landing page implementation:

- **Form Tests**: Single email field with honeypot protection
- **Analytics Tests**: Umami event tracking for all user interactions
- **Mobile Tests**: Responsive design with persistent bottom CTA
- **Integration Tests**: Full API functionality in workerd environment

## Troubleshooting

### Common Issues

1. **Test timeout errors**:

   ```bash
   # Increase timeout for slow systems
   pnpm exec playwright test --timeout=60000
   ```

2. **Wrangler dev server startup issues**:

   ```bash
   # Ensure build is up to date
   pnpm build
   # Check wrangler version
   npx wrangler --version
   ```

3. **Browser installation issues**:
   ```bash
   # Reinstall Playwright browsers
   pnpm exec playwright install --force
   ```

### Debug Mode

Run tests with debug output:

```bash
# Playwright debug mode
DEBUG=pw:api pnpm test:e2e

# Headed browser with slow motion
pnpm exec playwright test --headed --slow-mo=1000
```

## CI/CD Integration

For automated testing in CI/CD pipelines:

```bash
# Install dependencies
pnpm install
pnpm exec playwright install --with-deps

# Build and test
pnpm build
pnpm test:all
```

## Test Maintenance

When updating the landing page:

1. **Form Changes**: Update `e2e/email-collection.spec.ts`
2. **Analytics Changes**: Update `e2e/analytics-interactions.spec.ts`
3. **Mobile Changes**: Update `e2e/mobile-behavior.spec.ts`
4. **API Changes**: Update `tests/integration/wrangler-pages.test.ts`

Always run the full test suite after making changes:

```bash
pnpm build && pnpm test:all
```
