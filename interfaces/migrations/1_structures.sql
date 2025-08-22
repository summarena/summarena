-- SQLite Database Schema for interfaces/src/defs.rs structures
-- Only structures with URI fields become tables, others are nested as JSON

-- Live source specifications
CREATE TABLE live_source_specs (
    uri TEXT PRIMARY KEY
);

-- Input items with optional vision data
CREATE TABLE input_items (
    uri TEXT PRIMARY KEY,
    live_source_uri TEXT NOT NULL,  -- Reference to live source
    text TEXT NOT NULL,
    vision BLOB,  -- Optional binary data for vision
    FOREIGN KEY (live_source_uri) REFERENCES live_source_specs(uri)
);

-- Digest preferences
CREATE TABLE digest_preferences (
    uri TEXT PRIMARY KEY,
    description TEXT NOT NULL
);

-- Digest datasets with input item URIs as JSON array
CREATE TABLE digest_datasets (
    uri TEXT PRIMARY KEY,
    input_item_uris TEXT NOT NULL  -- JSON array of strings
);

-- Digest model specifications
CREATE TABLE digest_model_specs (
    uri TEXT PRIMARY KEY
);

-- Digest attempts with nested DigestOutput containing DigestSelectedItems with InputItemReferences
CREATE TABLE digest_attempts (
    uri TEXT PRIMARY KEY,
    dataset_uri TEXT NOT NULL,
    model_uri TEXT NOT NULL,
    output TEXT NOT NULL,  -- JSON: {selected_items: [{input_item_uri: "...", references: [{text_start_index: n, text_end_index: n}, ...]}, ...], text: "..."}
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (dataset_uri) REFERENCES digest_datasets(uri),
    FOREIGN KEY (model_uri) REFERENCES digest_model_specs(uri)
);
