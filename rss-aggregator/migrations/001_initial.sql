-- Create feeds table
CREATE TABLE IF NOT EXISTS feeds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url TEXT NOT NULL UNIQUE,
    title TEXT,
    description TEXT,
    last_fetch_time TIMESTAMPTZ,
    last_successful_fetch TIMESTAMPTZ,
    update_frequency_hours INTEGER,
    error_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    etag TEXT,
    last_modified TEXT
);

-- Create feed_entries table
CREATE TABLE IF NOT EXISTS feed_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feed_id UUID NOT NULL,
    guid TEXT,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    content TEXT,
    author TEXT,
    published_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    tags JSONB, -- JSON array
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_processed TIMESTAMPTZ,
    FOREIGN KEY (feed_id) REFERENCES feeds (id) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_feeds_active ON feeds (is_active);
CREATE INDEX IF NOT EXISTS idx_feeds_last_fetch ON feeds (last_fetch_time);
CREATE INDEX IF NOT EXISTS idx_feeds_url ON feeds (url);

CREATE INDEX IF NOT EXISTS idx_entries_feed_id ON feed_entries (feed_id);
CREATE INDEX IF NOT EXISTS idx_entries_guid ON feed_entries (guid);
CREATE INDEX IF NOT EXISTS idx_entries_url ON feed_entries (url);
CREATE INDEX IF NOT EXISTS idx_entries_published ON feed_entries (published_at);
CREATE INDEX IF NOT EXISTS idx_entries_processed ON feed_entries (last_processed);

-- Create unique constraint for preventing duplicate entries
CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_unique_guid_feed ON feed_entries (feed_id, guid) WHERE guid IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_unique_url_feed ON feed_entries (feed_id, url);