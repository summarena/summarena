import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { spawn, ChildProcess } from 'child_process';

describe('Wrangler Pages Integration Tests', () => {
  let wranglerProcess: ChildProcess;
  const baseUrl = 'http://localhost:8788';

  beforeAll(async () => {
    // Start Wrangler Pages dev server
    console.log('Starting Wrangler Pages dev server...');

    wranglerProcess = spawn(
      'npx',
      [
        'wrangler',
        'pages',
        'dev',
        'dist',
        '--port',
        '8788',
        '--kv',
        'EMAILS',
        '--binding',
        'GSHEET_WEBHOOK_URL=https://httpbin.org/post',
      ],
      {
        stdio: ['ignore', 'pipe', 'pipe'],
        detached: false,
      }
    );

    // Wait for server to start
    await new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Wrangler dev server failed to start within 30 seconds'));
      }, 30000);

      wranglerProcess.stdout?.on('data', data => {
        const output = data.toString();
        console.log('Wrangler output:', output);
        if (output.includes('Ready on') || output.includes('localhost:8788')) {
          clearTimeout(timeout);
          resolve();
        }
      });

      wranglerProcess.stderr?.on('data', data => {
        console.error('Wrangler error:', data.toString());
      });

      wranglerProcess.on('error', error => {
        clearTimeout(timeout);
        reject(error);
      });

      wranglerProcess.on('exit', code => {
        if (code !== 0) {
          clearTimeout(timeout);
          reject(new Error(`Wrangler process exited with code ${code}`));
        }
      });
    });

    // Give it a moment to fully initialize
    await new Promise(resolve => setTimeout(resolve, 2000));
  }, 60000); // 60 second timeout for startup

  afterAll(async () => {
    if (wranglerProcess) {
      console.log('Stopping Wrangler Pages dev server...');
      wranglerProcess.kill('SIGTERM');

      // Wait for process to exit
      await new Promise<void>(resolve => {
        wranglerProcess.on('exit', () => resolve());
        // Force kill after 5 seconds if it doesn't exit gracefully
        setTimeout(() => {
          wranglerProcess.kill('SIGKILL');
          resolve();
        }, 5000);
      });
    }
  });

  it('should serve the homepage with updated form', async () => {
    const response = await fetch(`${baseUrl}/`);
    expect(response.status).toBe(200);

    const html = await response.text();
    expect(html).toContain('SummArena');
    expect(html).toContain('lead-form'); // Updated form ID
    expect(html).toContain('Start free'); // Updated CTA text
    expect(html).toContain('One AI brief keeps you current'); // Updated hero copy
  });

  it('should successfully subscribe a new email via API (minimal form)', async () => {
    const uniqueEmail = `integration-test-${Date.now()}@example.com`;
    const formData = new FormData();
    formData.append('email', uniqueEmail);
    formData.append('company', ''); // Empty honeypot field

    const response = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData,
    });

    expect(response.status).toBe(200);

    const result = await response.json();
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });
  });

  it('should handle honeypot protection - silently succeed for bots', async () => {
    const uniqueEmail = `bot-test-${Date.now()}@example.com`;
    const formData = new FormData();
    formData.append('email', uniqueEmail);
    formData.append('company', 'Bot Company Inc'); // Filled honeypot field

    const response = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData,
    });

    expect(response.status).toBe(200);

    const result = await response.json();
    expect(result).toEqual({
      ok: true,
      status: 'subscribed', // Should return success but not actually store
    });

    // Verify the email was NOT actually stored by trying again without honeypot
    const formData2 = new FormData();
    formData2.append('email', uniqueEmail);
    formData2.append('company', ''); // Empty honeypot

    const response2 = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData2,
    });

    expect(response2.status).toBe(200);
    const result2 = await response2.json();
    // If honeypot worked, this should be 'subscribed' not 'already_subscribed'
    expect(result2.status).toBe('subscribed');
  });

  it('should handle already subscribed email', async () => {
    const email = `already-subscribed-${Date.now()}@example.com`;

    // First subscription
    const formData1 = new FormData();
    formData1.append('email', email);
    formData1.append('company', ''); // Empty honeypot

    const response1 = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData1,
    });

    expect(response1.status).toBe(200);
    const result1 = await response1.json();
    expect(result1.status).toBe('subscribed');

    // Second subscription (should be already_subscribed)
    const formData2 = new FormData();
    formData2.append('email', email);
    formData2.append('company', ''); // Empty honeypot

    const response2 = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData2,
    });

    expect(response2.status).toBe(200);
    const result2 = await response2.json();
    expect(result2).toEqual({
      ok: true,
      status: 'already_subscribed',
    });
  });

  it('should reject invalid email addresses', async () => {
    const formData = new FormData();
    formData.append('email', 'invalid-email-format');
    formData.append('company', ''); // Empty honeypot

    const response = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData,
    });

    expect(response.status).toBe(400);

    const result = await response.json();
    expect(result).toEqual({
      ok: false,
      error: 'invalid_email',
    });
  });

  it('should handle JSON payloads', async () => {
    const response = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        email: `json-test-${Date.now()}@example.com`,
        company: '', // Empty honeypot
      }),
    });

    expect(response.status).toBe(200);

    const result = await response.json();
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });
  });

  it('should normalize email case', async () => {
    const timestamp = Date.now();
    const formData = new FormData();
    formData.append('email', `UPPERCASE-${timestamp}@EXAMPLE.COM`);
    formData.append('company', ''); // Empty honeypot

    const response = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData,
    });

    expect(response.status).toBe(200);

    const result = await response.json();
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });

    // Try the same email in lowercase - should be already subscribed
    const formData2 = new FormData();
    formData2.append('email', `uppercase-${timestamp}@example.com`);
    formData2.append('company', ''); // Empty honeypot

    const response2 = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData2,
    });

    expect(response2.status).toBe(200);

    const result2 = await response2.json();
    expect(result2).toEqual({
      ok: true,
      status: 'already_subscribed',
    });
  });

  it('should handle CORS preflight requests', async () => {
    const response = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'OPTIONS',
    });

    expect(response.status).toBe(200);
    expect(response.headers.get('Access-Control-Allow-Origin')).toBe('*');
    expect(response.headers.get('Access-Control-Allow-Methods')).toContain('POST');
  });

  it('should return proper error for missing email', async () => {
    const formData = new FormData();
    formData.append('company', ''); // Empty honeypot but no email

    const response = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData,
    });

    expect(response.status).toBe(400);

    const result = await response.json();
    expect(result).toEqual({
      ok: false,
      error: 'invalid_email',
    });
  });

  it('should serve privacy and terms pages', async () => {
    const privacyResponse = await fetch(`${baseUrl}/privacy`);
    expect(privacyResponse.status).toBe(200);
    const privacyHtml = await privacyResponse.text();
    expect(privacyHtml).toContain('Privacy Policy');

    const termsResponse = await fetch(`${baseUrl}/terms`);
    expect(termsResponse.status).toBe(200);
    const termsHtml = await termsResponse.text();
    expect(termsHtml).toContain('Terms of Service');
  });

  it('should serve sample download link (even if file missing)', async () => {
    // This will 404 until actual file is added, but should not crash
    const response = await fetch(`${baseUrl}/sample-brief.pdf`);
    // Either 200 (if file exists) or 404 (if not) - both are acceptable
    expect([200, 404]).toContain(response.status);
  });

  it('should handle metadata correctly', async () => {
    const response = await fetch(`${baseUrl}/`);
    const html = await response.text();

    // Check updated title and meta description
    expect(html).toContain('SummArena — AI research brief with citations');
    expect(html).toContain(
      'One daily AI brief from your sources: arXiv, newsletters, RSS, YouTube. Linked citations. Free during pilot.'
    );

    // Check structured data
    expect(html).toContain('application/ld+json');
    expect(html).toContain('SoftwareApplication');
    expect(html).toContain('FAQPage');
  });

  it('should maintain backward compatibility for role/interests fields if sent', async () => {
    // Even though UI doesn't have these fields, API should still accept them
    const uniqueEmail = `backward-compat-${Date.now()}@example.com`;
    const formData = new FormData();
    formData.append('email', uniqueEmail);
    formData.append('role', 'Legacy Tester'); // Should be ignored but not error
    formData.append('interests', 'Legacy interests'); // Should be ignored but not error
    formData.append('company', ''); // Empty honeypot

    const response = await fetch(`${baseUrl}/api/subscribe`, {
      method: 'POST',
      body: formData,
    });

    expect(response.status).toBe(200);
    const result = await response.json();
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });
  });
});
