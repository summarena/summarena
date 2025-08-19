interface Env {
  EMAILS: any;
  GSHEET_WEBHOOK_URL?: string;
}

export const onRequestPost = async ({
  request,
  env,
  waitUntil,
}: {
  request: Request;
  env: Env;
  waitUntil: (promise: Promise<any>) => void;
}) => {
  const json = (d: unknown, s = 200) =>
    new Response(JSON.stringify(d), {
      status: s,
      headers: {
        'Content-Type': 'application/json',
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

    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      return json({ ok: false, error: 'invalid_email' }, 400);
    }

    const key = `email:${email}`;
    if (await env.EMAILS.get(key)) {
      return json({ ok: true, status: 'already_subscribed' });
    }

    const userData = { email, role, interests, ts: new Date().toISOString() };
    await env.EMAILS.put(key, JSON.stringify(userData), { metadata: { ts: userData.ts } });

    if (env.GSHEET_WEBHOOK_URL) {
      waitUntil(
        fetch(env.GSHEET_WEBHOOK_URL, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(userData),
        }).catch(console.error)
      );
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
