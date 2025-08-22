# RSS Aggregator

A high-performance RSS feed aggregator built in Rust that can handle many feeds efficiently. This system integrates with the existing interfaces crate and provides a robust foundation for RSS feed processing.

## Features

- **Feed Management**: Registry of RSS feed URLs with metadata tracking
- **Smart Fetching**: HTTP optimizations including compression, caching, and rate limiting
- **Robust Parsing**: Handles RSS/Atom formats with encoding normalization
- **Deduplication**: Prevents duplicate entries using GUIDs and URLs
- **Error Handling**: Exponential backoff and retry logic for failed feeds
- **Robots.txt Compliance**: Respects robots.txt when configured
- **Database Integration**: PostgreSQL-based storage with migrations
- **Interface Integration**: Compatible with the existing interfaces crate

## Architecture

### Core Components

1. **Feed Manager** (`feed_manager.rs`)
   - Maintains registry of RSS feeds
   - Tracks metadata (last fetch time, error counts, etc.)
   - Manages feed scheduling and validation
   - Database operations for feeds

2. **Fetcher/Crawler** (`fetcher.rs`)
   - HTTP client optimized for RSS feeds
   - Handles redirects, compression, user agents
   - Implements rate limiting and retry logic
   - Respects robots.txt when enabled
   - Conditional fetching using ETags and Last-Modified headers

3. **Parser** (`parser.rs`)
   - Normalizes RSS/Atom formats
   - Extracts structured metadata
   - Handles encoding issues
   - Deduplicates entries using GUIDs and URLs

4. **Aggregator** (`aggregator.rs`)
   - Main orchestrator combining all components
   - Implements the `Ingester` trait from interfaces
   - Provides high-level API for RSS operations

### Data Flow

1. Feed URLs are registered in the Feed Manager
2. Fetcher retrieves RSS content with HTTP optimizations
3. Parser normalizes and extracts structured data
4. Entries are deduplicated and stored as `InputItem`s
5. Other components can process the resulting dataset

## Integration with Interfaces

The RSS aggregator integrates with the existing interfaces crate:

- **`InputItem`**: RSS entries are converted to this format for downstream processing
- **`LiveSourceSpec`**: RSS feeds are represented as live sources
- **`Ingester`**: The aggregator implements this trait for feed monitoring

## Usage

### Basic Setup

```rust
use rss_aggregator::{RssAggregator, FetchConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the aggregator
    let config = FetchConfig::default();
    let aggregator = RssAggregator::new("postgresql://rss_user:rss_password@localhost:5432/rss_aggregator", config).await?;
    
    // Add feeds
    let feed_id = aggregator.add_feed("https://example.com/rss".to_string()).await?;
    
    // Fetch all feeds
    let fetched_count = aggregator.fetch_all_feeds().await?;
    
    Ok(())
}
```

### Configuration

The `FetchConfig` allows customization of fetching behavior:

```rust
let config = FetchConfig {
    user_agent: "MyApp RSS Reader/1.0".to_string(),
    timeout_seconds: 30,
    max_retries: 3,
    respect_robots_txt: true,
    max_feed_size_mb: 10,
    ..Default::default()
};
```

## Database Schema

The system uses PostgreSQL with the following tables:

- **feeds**: Feed metadata and configuration
- **feed_entries**: Individual RSS entries with full content

Migrations are located in the `migrations/` directory.

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client with compression and redirect support
- `feed-rs`: RSS/Atom parsing
- `sqlx`: Database operations with compile-time checked queries
- `chrono`: Date/time handling
- `uuid`: Unique identifiers
- `serde`: Serialization
- `tracing`: Structured logging
- `backoff`: Exponential backoff for retries

## Running

### Prerequisites

You need PostgreSQL running. Use the included Makefile to start a PostgreSQL container:

```bash
# Start PostgreSQL container
make postgres

# Stop PostgreSQL container (when done)
make postgres-stop

# Clean up container and data (if needed)
make postgres-clean
```

### Running the Application

```bash
# Set database URL
export DATABASE_URL="postgresql://rss_user:rss_password@localhost:5432/rss_aggregator"

# Run the application
cargo run

# Or build and run
cargo build --release
./target/release/rss-aggregator
```

## Testing

The project includes comprehensive integration tests that demonstrate the full RSS ingestion pipeline.

### Running Integration Tests

The integration tests fetch real RSS feeds and verify the complete data flow from fetching to database storage.

```bash
# Ensure PostgreSQL is running
make postgres

# Run the WSJ RSS feed integration test
DATABASE_URL="postgresql://rss_user:rss_password@localhost:5432/rss_aggregator" cargo test test_wsj_rss_feed_integration -- --nocapture

# Or run all tests
DATABASE_URL="postgresql://rss_user:rss_password@localhost:5432/rss_aggregator" cargo test
```

### What the Integration Test Does

The `test_wsj_rss_feed_integration` test:

1. **Clears and recreates the database schema**
2. **Adds the WSJ World News RSS feed** (`https://feeds.content.dowjones.io/public/rss/RSSWorldNews`)
3. **Fetches and parses the live RSS feed** (typically ~70+ current news articles)
4. **Stores all entries in the PostgreSQL database**
5. **Verifies data integrity** by querying the stored entries
6. **Displays the first 10 news articles** with titles, URLs, descriptions, authors, and publication dates

This provides a complete end-to-end verification that the RSS aggregator can:
- Connect to external RSS feeds
- Parse real-world RSS/XML content
- Handle PostgreSQL database operations
- Process and store structured data correctly

### Sample Test Output

```
=== RSS FEED ITEMS FROM DATABASE ===
Found 72 entries (showing first 10):

--- Entry 1 ---
Title: Putin Calls Zelensky the West's Illegitimate Puppet. Can He Talk Peace With Him?
URL: https://www.wsj.com/world/putin-zelensky-meeting-challenges-ed44eb54
Description: President Trump seeks to broker a meeting of the two leaders to end Europe's most destructive war in generations.
Author: Thomas Grove
Published: 2025-08-20 01:00:00 UTC

[... more entries ...]
```

## Future Enhancements

- Scheduler for periodic feed fetching
- REST API for feed management
- Content extraction from linked articles
- Feed auto-discovery
- Metrics and monitoring
- Distributed processing support
- Feed health scoring
- Custom parsing rules per feed