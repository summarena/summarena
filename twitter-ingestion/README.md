# Twitter Ingestion

This module provides Twitter ingestion functionality following the interfaces pattern, designed to fetch tweets from configured Twitter handles using the Twitter API v2 with Bearer token authentication.

## Features

- Fetches latest tweets from configured Twitter handles
- Supports multiple Twitter handles per ingester instance  
- Integrates with the interfaces storage system
- Simple Bearer token authentication (read-only access)
- Environment variable-based configuration with optional database storage
- Direct HTTP API calls using reqwest
- Date filtering to avoid fetching old tweets (respects last_sync_date)

## Configuration

The Twitter ingester accepts Twitter handles in URI format:
```
twitter://user/handle
```

For multiple handles:
```
twitter://user/handle1,handle2,handle3
```

The '@' prefix is automatically stripped if present, and whitespace is ignored.

## Usage

The module implements the `Ingester` trait and can be used with the `interfaces::state` system:

```rust
use twitter_ingestion::{TwitterIngester, TwitterIngesterConfig};
use chrono::{DateTime, Utc};

// Parse handles from URI
let handles = TwitterIngesterConfig::parse_handles_from_uri("twitter://user/dawnsongtweets")?;

// Create config without date filtering
let config = TwitterIngesterConfig::new(handles.clone(), bearer_token.clone());

// Or create config with date filtering
let last_sync = DateTime::parse_from_rfc3339("2024-01-15T10:30:45.000Z")?.with_timezone(&Utc);
let config_with_date = TwitterIngesterConfig::new_with_last_sync_date(handles, bearer_token, Some(last_sync));

let ingester = TwitterIngester::with_config(config_with_date);
let tweets = ingester.fetch_tweets(&source).await?;
```

## Twitter API Requirements

- Twitter API v2 access with Bearer token
- Bearer token provides read-only access to public tweets
- Much simpler than OAuth 1.0a - just one token needed
- Rate limits: The module respects Twitter API rate limits with appropriate delays

## Getting a Bearer Token

1. Apply for a Twitter Developer account at https://developer.twitter.com
2. Create an app in the Twitter Developer Portal
3. Go to your app's "Keys and tokens" section
4. Generate a Bearer Token (this is all you need for read-only access)

## Environment Variables

Required environment variable for Twitter API access:
- `TWITTER_BEARER_TOKEN`: Your Twitter API Bearer token

Optional environment variables:
- `TWITTER_LAST_SYNC_DATE`: RFC3339 formatted date (e.g., `2024-01-15T10:30:45.000Z`) to only fetch tweets newer than this date
- `DATABASE_URL`: PostgreSQL connection string for persistent sync date storage

## Database Support

The ingester supports optional PostgreSQL database integration for persistent storage:

- **Handle Tracking**: Automatically stores Twitter handles for tracking purposes
- **Last Sync Persistence**: Tracks last sync date per handle for incremental fetching  
- **Schema Management**: Automatically creates required tables on first run
- **Fallback Support**: Falls back to environment variables if database unavailable
- **Bearer Token Security**: Bearer tokens are always read from environment variables, never stored in database

When `DATABASE_URL` is provided, the ingester will:
1. Connect to PostgreSQL and setup the required schema
2. Use the later of database sync date or `TWITTER_LAST_SYNC_DATE` if both exist
3. Fall back to `TWITTER_LAST_SYNC_DATE` if no database entry exists
4. Update the last sync timestamp after successful ingestion

Database schema:
```sql
CREATE TABLE twitter_last_sync (
    handle VARCHAR(255) PRIMARY KEY,
    last_sync_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

## Tweet Data Format

Each ingested tweet is stored as an `InputItem` with:
- URI: `twitter://tweet/{tweet_id}`
- Text: Formatted tweet content including author, timestamp, content, URL, and metrics
- Vision: Currently unused (None)

## Date Filtering

The ingester supports date filtering to avoid fetching old tweets:
- Uses the `start_time` parameter in Twitter API requests
- Only fetches tweets newer than the `last_sync_date`
- Date can be set via environment variable `TWITTER_LAST_SYNC_DATE`
- If no date is provided, fetches the most recent tweets (up to API limits)

## Rate Limiting

The ingester implements conservative rate limiting:
- 5 minutes between successful ingestion cycles
- 15 minutes delay on errors
- Maximum 10 tweets per user per request (to stay within API limits)

## API Endpoints Used

- `GET /2/users/by/username/{username}` - Get user ID from username
- `GET /2/users/{id}/tweets` - Get tweets for a user
  - With optional `start_time` parameter for date filtering
  - Includes `tweet.fields=created_at,author_id,public_metrics` for metadata

## Example Usage with LiveSourceSpec

```rust
// The source.uri should contain the Twitter URI with a single handle
let source = LiveSourceSpec {
    uri: "twitter://user/dawnsongtweets".to_string(),
};

// The ingester will parse this automatically
TwitterIngester::watch(&source).await?;
```

## Example Tweet Output

```
@dawnsongtweets - 2024-01-15T10:30:45.000Z
This is an example tweet content
https://twitter.com/dawnsongtweets/status/1234567890
Likes: 42, Retweets: 12, Replies: 8, Quotes: 3
```

## Input Format Examples

All of these formats work:
- `"twitter://user/dawnsongtweets"` (single handle)
- `"twitter://user/handle1,handle2,handle3"` (multiple handles)
- `"twitter://user/@dawnsongtweets"` (@ prefix stripped)
- `"twitter://user/  dawnsongtweets  "` (whitespace is trimmed)

## Error Handling

The ingester handles various error scenarios:
- Missing Bearer token (waits 5 minutes)
- Invalid Twitter handle URI (waits 5 minutes)  
- API rate limits (waits 15 minutes)
- User not found or unauthorized (fails with error)
- Network errors (waits 15 minutes)