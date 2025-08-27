use crate::types::{Ingester, WatchRest, LiveSourceSpec, FetchConfig};
use crate::sources::RssFeedSource;
use crate::traits::PullFeed;
use std::future::Future;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, error};

pub struct RssIngester;

impl Ingester for RssIngester {
    fn watch(source: &LiveSourceSpec) -> impl Future<Output = Result<WatchRest>> {
        async move {
            info!("Starting RSS ingester watch for source: {}", source.uri);
            
            // Create a default fetch config
            let fetch_config = FetchConfig::default();
            
            // Create RSS feed source
            let feed_id = Uuid::new_v4();
            let mut rss_source = RssFeedSource::new(
                feed_id,
                source.uri.clone(),
                fetch_config,
                None, // Use default poll interval
            );
            
            // Pull items from the RSS feed
            match rss_source.pull().await {
                Ok(items) => {
                    info!("Successfully pulled {} items from RSS feed", items.len());
                    
                    // Ingest each item using the interfaces state module
                    for item in items {
                        if let Err(e) = interfaces::state::ingest(&item).await {
                            error!("Failed to ingest item {}: {}", item.uri, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to pull items from RSS feed {}: {}", source.uri, e);
                }
            }
            
            // Return the recommended wait time based on the source's poll interval
            let poll_interval = rss_source.poll_interval_ms();
            Ok(WatchRest {
                wait_at_least_ms: poll_interval as u32,
            })
        }
    }
}