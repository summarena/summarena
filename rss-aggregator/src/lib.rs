pub mod types;
pub mod feed_manager;
pub mod fetcher;
pub mod parser;
pub mod aggregator;
pub mod state;
pub mod digest;
pub mod traits;
pub mod sources;
pub mod aggregators;
pub mod pipeline;
pub mod processing;
pub mod user_manager;
pub mod utils;
pub mod llm_adapter;

pub use types::*;
pub use feed_manager::FeedManager;
pub use fetcher::Fetcher;
pub use parser::FeedParser;
pub use aggregator::{RssAggregator, RssIngester};
pub use state::RssState;
pub use digest::RssDigestModel;

// Re-export new architecture components
pub use traits::{PullFeed, Aggregator, SourceMetadata, AggregatedOutput, AggregatorConfig};
pub use sources::{RssFeedSource, WsjFeedSource};
pub use aggregators::TimeBucketAggregator;
pub use pipeline::{IngestionPipeline, PipelineBuilder};
pub use processing::{ProcessingStage, ProcessingInput, ProcessingOutput, ProcessedItem, RelevanceStage, SummarizationStage, FilterStage};
pub use user_manager::{UserAggregatorManager, UserAggregatorBuilder, ManagerStats, BulkCreateResult};
pub use llm_adapter::{LlmAdapter, LlmAdapterRegistry, LlmAdapterBuilder, MockLlmAdapter, CompiledLlmPreferences};