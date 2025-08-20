-- Complete RSS Aggregator Database Schema
-- This migration creates all tables and indexes needed for the RSS aggregation system

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

-- Create feed_entries table for raw RSS entries
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

-- Create input_items table for normalized InputItem storage
-- This table stores RSS items in the format expected by the interfaces crate
CREATE TABLE IF NOT EXISTS input_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feed_id UUID NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    uri VARCHAR NOT NULL UNIQUE, -- URI from InputItem (typically the RSS item URL)
    title VARCHAR NOT NULL,
    description TEXT,
    content TEXT,
    vision_data BYTEA DEFAULT '', -- Vision data from InputItem (typically empty for RSS)
    text_content TEXT NOT NULL, -- Full text content as formatted by InputItem
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for feeds table
CREATE INDEX IF NOT EXISTS idx_feeds_active ON feeds (is_active);
CREATE INDEX IF NOT EXISTS idx_feeds_last_fetch ON feeds (last_fetch_time);
CREATE INDEX IF NOT EXISTS idx_feeds_url ON feeds (url);

-- Create indexes for feed_entries table
CREATE INDEX IF NOT EXISTS idx_entries_feed_id ON feed_entries (feed_id);
CREATE INDEX IF NOT EXISTS idx_entries_guid ON feed_entries (guid);
CREATE INDEX IF NOT EXISTS idx_entries_url ON feed_entries (url);
CREATE INDEX IF NOT EXISTS idx_entries_published ON feed_entries (published_at);
CREATE INDEX IF NOT EXISTS idx_entries_processed ON feed_entries (last_processed);

-- Create unique constraints for preventing duplicate entries
CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_unique_guid_feed ON feed_entries (feed_id, guid) WHERE guid IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_unique_url_feed ON feed_entries (feed_id, url);

-- Create indexes for input_items table
CREATE INDEX IF NOT EXISTS idx_input_items_feed_id ON input_items(feed_id);
CREATE INDEX IF NOT EXISTS idx_input_items_created_at ON input_items(created_at);
CREATE INDEX IF NOT EXISTS idx_input_items_uri ON input_items(uri);

-- Full text search index for content searching in input_items
CREATE INDEX IF NOT EXISTS idx_input_items_text_search ON input_items USING gin(to_tsvector('english', coalesce(title, '') || ' ' || coalesce(description, '') || ' ' || coalesce(content, '')));

-- Update trigger function for input_items table to maintain updated_at timestamp
CREATE OR REPLACE FUNCTION update_input_items_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for input_items updated_at maintenance
CREATE TRIGGER trigger_input_items_updated_at
    BEFORE UPDATE ON input_items
    FOR EACH ROW
    EXECUTE FUNCTION update_input_items_updated_at();