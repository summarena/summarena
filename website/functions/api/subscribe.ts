interface Env {
  EMAILS: any; // KV Namespace
  GSHEET_WEBHOOK_URL?: string;
}

export const onRequestPost = async ({ request, env }: { request: Request; env: Env }) => {
  const json = (d: unknown, s = 200) =>
    new Response(JSON.stringify(d), {
      status: s,
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'no-store',
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'POST',
        'Access-Control-Allow-Headers': 'Content-Type',
      },
    });

  try {
    const ct = request.headers.get('content-type') || '';
    const body = ct.includes('application/json')
      ? await request.json()
      : Object.fromEntries(await request.formData());

    const raw = String((body as any).email || '')
      .trim()
      .toLowerCase();
    const valid = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(raw);

    if (!valid) {
      return json({ ok: false, error: 'invalid_email' }, 400);
    }

    const key = `email:${raw}`;

    // Check if email already exists
    if (await env.EMAILS.get(key)) {
      return json({ ok: true, status: 'already_subscribed' });
    }

    const ip = request.headers.get('cf-connecting-ip') || '';
    const ts = new Date().toISOString();

    // Store in KV
    await env.EMAILS.put(key, JSON.stringify({ email: raw, ts, ip }), {
      metadata: { ts },
    });

    // Mirror to Google Sheets webhook (fire-and-forget)
    const webhook = env.GSHEET_WEBHOOK_URL;
    if (webhook) {
      fetch(webhook, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: raw, ts, ip }),
      }).catch(() => {}); // Silent fail for webhook
    }

    return json({ ok: true, status: 'subscribed' });
  } catch (error) {
    console.error('Subscribe endpoint error:', error);
    return json({ ok: false, error: 'server_error' }, 500);
  }
};

// Handle preflight requests for CORS
export const onRequestOptions = async () => {
  return new Response(null, {
    status: 200,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    },
  });
};
