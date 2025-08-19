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

    const email = String((body as any).email || '')
      .trim()
      .toLowerCase();
    const role = String((body as any).role || '').trim();
    const interests = String((body as any).interests || '').trim();

    const valid = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);

    if (!valid) {
      return json({ ok: false, error: 'invalid_email' }, 400);
    }

    const key = `email:${email}`;

    // Check if email already exists
    if (await env.EMAILS.get(key)) {
      return json({ ok: true, status: 'already_subscribed' });
    }

    const ts = new Date().toISOString();

    // Store in KV - save the actual form data
    const userData = {
      email,
      role,
      interests,
      ts,
    };

    await env.EMAILS.put(key, JSON.stringify(userData), {
      metadata: { ts },
    });

    // Mirror to Google Sheets webhook (fire-and-forget)
    const webhook = env.GSHEET_WEBHOOK_URL;
    if (webhook) {
      fetch(webhook, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
        },
        body: JSON.stringify(userData),
      })
      .then(response => {
        console.log('Webhook response status:', response.status);
        return response.text();
      })
      .then(text => {
        console.log('Webhook response body:', text);
      })
      .catch(error => {
        console.error('Webhook failed:', error);
      });
    } else {
      console.log('No webhook URL configured');
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
