use rss_aggregator::{
    types::*, 
    TimeBucketAggregator, Aggregator,
    RelevanceStage, SummarizationStage, FilterStage,
    ProcessingStage, ProcessingInput, ProcessedItem,
    DigestPreferences, DigestModelMemory,
    LlmAdapterRegistry, MockLlmAdapter
};
use std::collections::HashMap;
use tokio;
use tracing::info;
use tracing_subscriber;

#[tokio::test]
async fn test_processing_stages() -> Result<()> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    info!("Testing processing stages");
    
    // Create test input items
    let test_items = vec![
        InputItem {
            uri: "https://example.com/tech-news".to_string(),
            text: "Title: Revolutionary AI breakthrough\n\nDescription: Scientists develop new AI technology that will transform computing.\n\nContent: The new artificial intelligence system shows remarkable capabilities...".to_string(),
            vision: Vec::new(),
        },
        InputItem {
            uri: "https://example.com/sports-news".to_string(),
            text: "Title: Football championship results\n\nDescription: Local team wins big game.\n\nContent: The championship game was exciting with a final score of...".to_string(),
            vision: Vec::new(),
        },
        InputItem {
            uri: "https://example.com/business-news".to_string(),
            text: "Title: Stock market surge\n\nDescription: Technology stocks see major gains.\n\nContent: The technology sector led market gains today...".to_string(),
            vision: Vec::new(),
        },
    ];
    
    // Convert to ProcessedItems
    let processed_items: Vec<ProcessedItem> = test_items.into_iter()
        .map(|item| ProcessedItem::new(item))
        .collect();
    
    // Create user preferences
    let preferences = DigestPreferences {
        uri: "test-prefs".to_string(),
        description: "I'm interested in technology, AI, and business news".to_string(),
    };
    
    let memory = DigestModelMemory {
        text: "User has shown interest in tech and AI topics".to_string(),
    };
    
    // Create processing input
    let processing_input = ProcessingInput {
        items: processed_items,
        user_preferences: Some(preferences.clone()),
        user_memory: Some(memory.clone()),
        metadata: HashMap::new(),
    };
    
    // Test relevance stage
    let mut relevance_stage = RelevanceStage::new();
    let relevance_output = relevance_stage.process(processing_input).await?;
    
    info!("Relevance stage processed {} items", relevance_output.items.len());
    for item in &relevance_output.items {
        if let Some(score) = item.relevance_score {
            info!("Item relevance score: {:.2}", score);
        }
    }
    
    // Test summarization stage
    let mut summarization_stage = SummarizationStage::new();
    let summarization_input = ProcessingInput {
        items: relevance_output.items,
        user_preferences: Some(preferences.clone()),
        user_memory: Some(memory.clone()),
        metadata: relevance_output.metadata,
    };
    let summarization_output = summarization_stage.process(summarization_input).await?;
    
    info!("Summarization stage processed {} items", summarization_output.items.len());
    for item in &summarization_output.items {
        if let Some(summary) = &item.summary {
            info!("Item summary: {}", summary);
        }
    }
    
    // Test filter stage
    let mut filter_stage = FilterStage::new(0.3).with_max_items(10);
    let filter_input = ProcessingInput {
        items: summarization_output.items,
        user_preferences: Some(preferences),
        user_memory: Some(memory),
        metadata: summarization_output.metadata,
    };
    let filter_output = filter_stage.process(filter_input).await?;
    
    info!("Filter stage kept {} items", filter_output.items.len());
    
    // Assertions
    assert!(filter_output.items.len() > 0, "Should have some items after filtering");
    assert!(filter_output.items.len() <= 10, "Should respect max items limit");
    
    // Check that all items have required processing metadata
    for item in &filter_output.items {
        assert!(item.relevance_score.is_some(), "All items should have relevance scores");
        assert!(item.summary.is_some(), "All items should have summaries");
    }
    
    info!("Processing stages test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_time_bucket_aggregator() -> Result<()> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    info!("Testing time bucket aggregator");
    
    // Create aggregator with short duration for testing
    let mut aggregator = TimeBucketAggregator::new_with_duration("test-user".to_string(), 1); // 1 hour
    
    // Add test items
    let test_items = vec![
        InputItem {
            uri: "https://example.com/item1".to_string(),
            text: "Test item 1 content".to_string(),
            vision: Vec::new(),
        },
        InputItem {
            uri: "https://example.com/item2".to_string(),
            text: "Test item 2 content".to_string(),
            vision: Vec::new(),
        },
    ];
    
    for item in test_items {
        aggregator.add_item(item).await?;
    }
    
    // Check if ready to produce output
    let should_produce = aggregator.should_produce_output();
    info!("Aggregator should produce output: {}", should_produce);
    
    if should_produce {
        let output = aggregator.produce_output().await?;
        info!("Produced output with {} items", output.items.len());
        info!("Output summary: {:?}", output.summary);
        
        assert_eq!(output.user_id, "test-user");
        assert_eq!(output.aggregator_type, "time_bucket_1h");
        assert_eq!(output.items.len(), 2);
    }
    
    info!("Time bucket aggregator test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_llm_adapter() -> Result<()> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    info!("Testing LLM adapter");
    
    // Create LLM adapter registry
    let mut registry = LlmAdapterRegistry::new();
    
    // Add test adapters
    let mock_adapter1 = MockLlmAdapter::new("fast".to_string()).with_delay(50);
    let mock_adapter2 = MockLlmAdapter::new("detailed".to_string()).with_delay(100);
    
    registry.register_adapter(Box::new(mock_adapter1));
    registry.register_adapter(Box::new(mock_adapter2));
    
    // Test adapter listing
    let adapters = registry.list_adapters();
    info!("Registered adapters: {:?}", adapters);
    assert_eq!(adapters.len(), 2);
    
    // Test default adapter
    let default_adapter = registry.get_default_adapter().unwrap();
    info!("Default adapter: {}", default_adapter.adapter_name());
    
    // Test preferences compilation
    let preferences = DigestPreferences {
        uri: "test-prefs".to_string(),
        description: "I'm interested in technology, artificial intelligence, and business".to_string(),
    };
    
    let memory = DigestModelMemory {
        text: "User likes AI and tech topics".to_string(),
    };
    
    let compiled_prefs = default_adapter.compile_preferences(&preferences, &memory).await?;
    info!("Compiled preferences: keywords={:?}, topics={:?}", 
          compiled_prefs.keywords, compiled_prefs.topics);
    
    // Test relevance scoring
    let test_item = InputItem {
        uri: "https://example.com/ai-news".to_string(),
        text: "Revolutionary artificial intelligence breakthrough in technology sector".to_string(),
        vision: Vec::new(),
    };
    
    let relevance_score = default_adapter.score_relevance(&test_item, &compiled_prefs).await?;
    info!("Relevance score: {:.2}", relevance_score);
    assert!(relevance_score > 0.0, "Should have positive relevance score for relevant content");
    
    // Test summarization
    let summary = default_adapter.create_summary(&test_item, &compiled_prefs).await?;
    info!("Generated summary: {}", summary);
    assert!(!summary.is_empty(), "Summary should not be empty");
    
    // Test entity extraction
    let entities = default_adapter.extract_entities(&test_item).await?;
    info!("Extracted entities: {:?}", entities);
    
    // Test digest generation
    let items = vec![test_item];
    let digest = default_adapter.generate_digest(&items, &compiled_prefs).await?;
    info!("Generated digest: {}", digest);
    assert!(digest.contains("AI-Generated Digest"), "Should contain digest header");
    
    info!("LLM adapter test completed successfully!");
    Ok(())
}