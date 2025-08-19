import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock implementation for testing the function logic
const mockKV = {
  data: new Map<string, string>(),
  get: vi.fn().mockImplementation(async (key: string) => {
    return mockKV.data.get(key) || null;
  }),
  put: vi.fn().mockImplementation(async (key: string, value: string) => {
    mockKV.data.set(key, value);
  }),
};

const mockEnv = {
  EMAILS: mockKV,
  GSHEET_WEBHOOK_URL: 'https://httpbin.org/post',
};

const mockWaitUntil = vi.fn();

// Mock fetch for webhook testing
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

// Import the function after mocking
const { onRequestPost } = await import('../../functions/api/subscribe.ts');

describe('Subscribe API Endpoint', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockKV.data.clear();
    mockWaitUntil.mockClear();
    mockFetch.mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({ success: true }),
    });
  });

  it('should successfully subscribe a new email with full form data', async () => {
    const formData = new FormData();
    formData.append('email', 'test@example.com');
    formData.append('role', 'Product Manager');
    formData.append('interests', 'AI research, startup news');

    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      body: formData,
    });

    const response = await onRequestPost({ request, env: mockEnv, waitUntil: mockWaitUntil });
    const result = await response.json();

    expect(response.status).toBe(200);
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });

    expect(mockKV.put).toHaveBeenCalledWith(
      'email:test@example.com',
      expect.stringContaining('"email":"test@example.com"'),
      expect.any(Object)
    );

    // Check that the stored data includes role and interests
    const storedData = JSON.parse(mockKV.put.mock.calls[0][1]);
    expect(storedData).toMatchObject({
      email: 'test@example.com',
      role: 'Product Manager',
      interests: 'AI research, startup news',
      ts: expect.any(String),
    });

    expect(mockFetch).toHaveBeenCalledWith(
      'https://httpbin.org/post',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: expect.stringContaining('"role":"Product Manager"'),
      })
    );
  });

  it('should handle already subscribed emails', async () => {
    // Pre-populate KV with existing email
    await mockKV.put(
      'email:existing@example.com',
      JSON.stringify({
        email: 'existing@example.com',
        role: 'Researcher',
        interests: 'ML papers',
        ts: '2024-01-01T00:00:00.000Z',
      })
    );

    const formData = new FormData();
    formData.append('email', 'existing@example.com');

    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      body: formData,
    });

    const response = await onRequestPost({ request, env: mockEnv, waitUntil: mockWaitUntil });
    const result = await response.json();

    expect(response.status).toBe(200);
    expect(result).toEqual({
      ok: true,
      status: 'already_subscribed',
    });

    // Should not call put again for existing email
    expect(mockKV.put).toHaveBeenCalledTimes(1); // Only the initial setup call
  });

  it('should reject invalid email addresses', async () => {
    const formData = new FormData();
    formData.append('email', 'invalid-email');

    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      body: formData,
    });

    const response = await onRequestPost({ request, env: mockEnv, waitUntil: mockWaitUntil });
    const result = await response.json();

    expect(response.status).toBe(400);
    expect(result).toEqual({
      ok: false,
      error: 'invalid_email',
    });

    expect(mockKV.put).not.toHaveBeenCalled();
  });

  it('should handle JSON payload with all fields', async () => {
    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        email: 'json@example.com',
        role: 'Developer',
        interests: 'Web development',
      }),
    });

    const response = await onRequestPost({ request, env: mockEnv, waitUntil: mockWaitUntil });
    const result = await response.json();

    expect(response.status).toBe(200);
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });

    // Check webhook payload includes all fields
    expect(mockFetch).toHaveBeenCalledWith(
      'https://httpbin.org/post',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: expect.stringContaining('"role":"Developer"'),
      })
    );
  });

  it('should handle email-only submission', async () => {
    const formData = new FormData();
    formData.append('email', 'minimal@example.com');

    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      body: formData,
    });

    const response = await onRequestPost({ request, env: mockEnv, waitUntil: mockWaitUntil });
    const result = await response.json();

    expect(response.status).toBe(200);
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });

    // Check that empty role and interests are stored
    const storedData = JSON.parse(mockKV.put.mock.calls[0][1]);
    expect(storedData).toMatchObject({
      email: 'minimal@example.com',
      role: '',
      interests: '',
      ts: expect.any(String),
    });
  });

  it('should normalize email case', async () => {
    const formData = new FormData();
    formData.append('email', 'Test@EXAMPLE.COM');

    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      body: formData,
    });

    const response = await onRequestPost({ request, env: mockEnv, waitUntil: mockWaitUntil });
    const result = await response.json();

    expect(response.status).toBe(200);
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });

    expect(mockKV.put).toHaveBeenCalledWith(
      'email:test@example.com',
      expect.stringContaining('"email":"test@example.com"'),
      expect.any(Object)
    );
  });

  it('should handle missing email field', async () => {
    const formData = new FormData();
    // No email field

    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      body: formData,
    });

    const response = await onRequestPost({ request, env: mockEnv, waitUntil: mockWaitUntil });
    const result = await response.json();

    expect(response.status).toBe(400);
    expect(result).toEqual({
      ok: false,
      error: 'invalid_email',
    });
  });

  it('should work without webhook URL configured', async () => {
    const envWithoutWebhook = {
      EMAILS: mockKV,
      GSHEET_WEBHOOK_URL: undefined,
    };

    const formData = new FormData();
    formData.append('email', 'nowebhook@example.com');

    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      body: formData,
    });

    const response = await onRequestPost({ request, env: envWithoutWebhook, waitUntil: mockWaitUntil });
    const result = await response.json();

    expect(response.status).toBe(200);
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });

    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('should handle webhook failures gracefully', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));

    const formData = new FormData();
    formData.append('email', 'webhookfail@example.com');

    const request = new Request('http://localhost/api/subscribe', {
      method: 'POST',
      body: formData,
    });

    const response = await onRequestPost({ request, env: mockEnv, waitUntil: mockWaitUntil });
    const result = await response.json();

    // Should still succeed even if webhook fails
    expect(response.status).toBe(200);
    expect(result).toEqual({
      ok: true,
      status: 'subscribed',
    });
  });
});
