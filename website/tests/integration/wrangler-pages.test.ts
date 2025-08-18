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

  it('should serve the homepage', async () => {
    const response = await fetch(`${baseUrl}/`);
    expect(response.status).toBe(200);

    const html = await response.text();
    expect(html).toContain('SummArena');
    expect(html).toContain('contact-form');
  });

  it('should successfully subscribe a new email via API', async () => {
    const uniqueEmail = `integration-test-${Date.now()}@example.com`;
    const formData = new FormData();
    formData.append('email', uniqueEmail);
    formData.append('role', 'Tester');
    formData.append('interests', 'Testing, QA');

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

  it('should handle already subscribed email', async () => {
    const email = `already-subscribed-${Date.now()}@example.com`;

    // First subscription
    const formData1 = new FormData();
    formData1.append('email', email);

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
    // Don't add email field

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
});
