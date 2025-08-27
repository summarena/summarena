pub mod types;
pub mod fetcher;
pub mod parser;
pub mod traits;
pub mod sources;
pub mod rss_utils;
pub mod ingester;

// Re-export core types
pub use types::*;
pub use fetcher::Fetcher;
pub use parser::FeedParser;

// Re-export RSS ingester components
pub use traits::{PullFeed, SourceMetadata};
pub use sources::{RssFeedSource, WsjFeedSource};
pub use rss_utils::{url, time, feed};
pub use ingester::RssIngester;