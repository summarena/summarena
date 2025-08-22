use crate::traits::{PullFeed, AggregatedOutput};
use crate::processing::{ProcessingStage, ProcessedItem};
use crate::user_manager::UserAggregatorManager;
use crate::types::{InputItem, Result, AggregatorError, DigestPreferences, DigestModelMemory};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{info, warn, error, debug};

/// Main ingestion pipeline that coordinates sources, filtering, and aggregation
pub struct IngestionPipeline {
    sources: Vec<Box<dyn PullFeed>>,
    user_manager: Arc<UserAggregatorManager>,
    processing_stages: Vec<Box<dyn ProcessingStage>>,
    user_preferences: Arc<RwLock<HashMap<String, (DigestPreferences, DigestModelMemory)>>>,
    
    // Channels for pipeline stages
    raw_item_sender: mpsc::UnboundedSender<(String, InputItem)>, // (user_id, item)
    _processed_item_sender: mpsc::UnboundedSender<(String, ProcessedItem)>, // (user_id, processed_item)
    output_receiver: Arc<RwLock<mpsc::UnboundedReceiver<AggregatedOutput>>>,
    output_sender: mpsc::UnboundedSender<AggregatedOutput>,
    
    is_running: Arc<RwLock<bool>>,
}

impl IngestionPipeline {
    pub fn new() -> Self {
        let (raw_item_sender, raw_item_receiver) = mpsc::unbounded_channel();
        let (processed_item_sender, processed_item_receiver) = mpsc::unbounded_channel();
        let (output_sender, output_receiver) = mpsc::unbounded_channel();
        
        let user_manager = Arc::new(UserAggregatorManager::new());
        
        let pipeline = Self {
            sources: Vec::new(),
            user_manager: user_manager.clone(),
            processing_stages: Vec::new(),
            user_preferences: Arc::new(RwLock::new(HashMap::new())),
            raw_item_sender,
            _processed_item_sender: processed_item_sender.clone(),
            output_receiver: Arc::new(RwLock::new(output_receiver)),
            output_sender: output_sender.clone(),
            is_running: Arc::new(RwLock::new(false)),
        };
        
        // Start the processing pipeline workers
        pipeline.start_processing_worker(raw_item_receiver, processed_item_sender.clone());
        pipeline.start_aggregation_worker(processed_item_receiver);
        
        pipeline
    }
    
    /// Add a new content source to the pipeline
    pub fn add_source(&mut self, source: Box<dyn PullFeed>) {
        info!("Adding source to pipeline: {}", source.source_name());
        self.sources.push(source);
    }
    
    /// Add a processing stage to the pipeline
    pub fn add_processing_stage(&mut self, stage: Box<dyn ProcessingStage>) {
        info!("Adding processing stage to pipeline: {}", stage.stage_name());
        self.processing_stages.push(stage);
    }
    
    /// Set user preferences for personalized processing
    pub async fn set_user_preferences(
        &self,
        user_id: String,
        preferences: DigestPreferences,
        memory: DigestModelMemory,
    ) -> Result<()> {
        let mut user_prefs = self.user_preferences.write().await;
        user_prefs.insert(user_id.clone(), (preferences, memory));
        info!("Set preferences for user {}", user_id);
        Ok(())
    }
    
    /// Create a user aggregator through the user manager
    pub async fn create_user_aggregator(
        &self,
        user_id: String,
        aggregator_type: &str,
        config: Option<crate::traits::AggregatorConfig>,
    ) -> Result<()> {
        self.user_manager.create_user_aggregator(user_id, aggregator_type, config).await
    }
    
    /// Remove a user's aggregator
    pub async fn remove_user_aggregator(&self, user_id: &str) -> Result<()> {
        self.user_manager.remove_user_aggregator(user_id).await?;
        
        // Also remove their preferences
        let mut user_prefs = self.user_preferences.write().await;
        user_prefs.remove(user_id);
        
        Ok(())
    }
    
    /// Start the ingestion pipeline
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(AggregatorError::General("Pipeline is already running".to_string()));
        }
        *is_running = true;
        drop(is_running);
        
        info!("Starting ingestion pipeline with {} sources", self.sources.len());
        
        // Start source polling tasks
        for (i, _) in self.sources.iter().enumerate() {
            self.start_source_poller(i).await;
        }
        
        // Start output checker
        self.start_output_checker().await;
        
        Ok(())
    }
    
    /// Stop the ingestion pipeline
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Stopping ingestion pipeline");
        Ok(())
    }
    
    /// Get the next aggregated output if available
    pub async fn get_output(&self) -> Option<AggregatedOutput> {
        let mut receiver = self.output_receiver.write().await;
        receiver.try_recv().ok()
    }
    
    /// Manually trigger ingestion for all sources (useful for testing)
    pub async fn ingest_all_sources(&mut self) -> Result<usize> {
        let mut total_items = 0;
        
        for source in &mut self.sources {
            match source.pull().await {
                Ok(items) => {
                    info!("Pulled {} items from source: {}", items.len(), source.source_name());
                    
                    // Send items to all managed users
                    let managed_users = self.user_manager.get_managed_users().await;
                    for user_id in managed_users {
                        for item in &items {
                            if let Err(e) = self.raw_item_sender.send((user_id.clone(), item.clone())) {
                                warn!("Failed to send item to processing pipeline: {}", e);
                            }
                        }
                    }
                    
                    total_items += items.len();
                }
                Err(e) => {
                    error!("Failed to pull from source {}: {}", source.source_name(), e);
                }
            }
        }
        
        Ok(total_items)
    }
    
    /// Start a background task to poll a specific source
    async fn start_source_poller(&self, source_index: usize) {
        let _raw_item_sender = self.raw_item_sender.clone();
        let _user_manager = self.user_manager.clone();
        let is_running = self.is_running.clone();
        
        // We can't easily clone Box<dyn PullFeed>, so we'll use a different approach
        // In practice, you might want to use Arc<RwLock<dyn PullFeed>> or similar
        info!("Source poller for index {} would start here (simplified for demo)", source_index);
        
        tokio::spawn(async move {
            while *is_running.read().await {
                // This is where we'd poll the source and send items
                // For now, this is a placeholder
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }
    
    /// Start background task to check for ready aggregator outputs
    async fn start_output_checker(&self) {
        let user_manager = self.user_manager.clone();
        let output_sender = self.output_sender.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(30)); // Check every 30 seconds
            
            while *is_running.read().await {
                check_interval.tick().await;
                
                let aggregators_map = user_manager.get_aggregators_map();
                let mut aggregators = aggregators_map.write().await;
                let mut ready_outputs = Vec::new();
                
                // Check each aggregator for ready output
                for (user_id, aggregator) in aggregators.iter_mut() {
                    if aggregator.should_produce_output() {
                        match aggregator.produce_output().await {
                            Ok(output) => {
                                debug!("Produced output for user {}", user_id);
                                ready_outputs.push(output);
                            }
                            Err(e) => {
                                warn!("Failed to produce output for user {}: {}", user_id, e);
                            }
                        }
                    }
                }
                
                // Send outputs to the output channel
                for output in ready_outputs {
                    if let Err(e) = output_sender.send(output) {
                        warn!("Failed to send aggregated output: {}", e);
                    }
                }
            }
        });
    }
    
    /// Start the processing worker that runs items through processing stages
    fn start_processing_worker(
        &self,
        raw_item_receiver: mpsc::UnboundedReceiver<(String, InputItem)>,
        processed_item_sender: mpsc::UnboundedSender<(String, ProcessedItem)>,
    ) {
        let user_preferences = self.user_preferences.clone();
        
        tokio::spawn(async move {
            // Convert receiver to stream for easier processing
            let item_stream = UnboundedReceiverStream::new(raw_item_receiver);
            
            // Group items by user for batch processing
            let mut user_batches: HashMap<String, Vec<InputItem>> = HashMap::new();
            let mut batch_timer = interval(Duration::from_secs(5)); // Process batches every 5 seconds
            
            tokio::pin!(item_stream);
            
            loop {
                tokio::select! {
                    // Collect items into user batches
                    item_result = item_stream.next() => {
                        match item_result {
                            Some((user_id, item)) => {
                                debug!("Received item for processing: user={}, uri={}", user_id, item.uri);
                                user_batches.entry(user_id).or_insert_with(Vec::new).push(item);
                            }
                            None => break, // Stream ended
                        }
                    }
                    
                    // Process accumulated batches periodically
                    _ = batch_timer.tick() => {
                        if !user_batches.is_empty() {
                            let batches_to_process = std::mem::take(&mut user_batches);
                            
                            for (user_id, items) in batches_to_process {
                                if items.is_empty() {
                                    continue;
                                }
                                
                                debug!("Processing batch of {} items for user {}", items.len(), user_id);
                                
                                // Get user preferences
                                let user_prefs = user_preferences.read().await;
                                let prefs_and_memory = user_prefs.get(&user_id);
                                let (_preferences, _memory) = match prefs_and_memory {
                                    Some((p, m)) => (Some(p.clone()), Some(m.clone())),
                                    None => (None, None),
                                };
                                drop(user_prefs);
                                
                                // Convert InputItems to ProcessedItems and process them
                                let processed_items: Vec<ProcessedItem> = items.into_iter()
                                    .map(|item| ProcessedItem::new(item))
                                    .collect();
                                
                                // For now, just send the processed items without running through stages
                                // In a full implementation, you'd run through all processing stages here
                                for processed_item in processed_items {
                                    if let Err(e) = processed_item_sender.send((user_id.clone(), processed_item)) {
                                        warn!("Failed to send processed item: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    /// Start the worker that takes processed items and feeds them to aggregators
    fn start_aggregation_worker(&self, mut processed_item_receiver: mpsc::UnboundedReceiver<(String, ProcessedItem)>) {
        let user_manager = self.user_manager.clone();
        
        tokio::spawn(async move {
            while let Some((user_id, processed_item)) = processed_item_receiver.recv().await {
                debug!("Adding processed item to aggregator for user {}: {}", user_id, processed_item.item.uri);
                
                let aggregators_map = user_manager.get_aggregators_map();
                let mut aggregators = aggregators_map.write().await;
                
                if let Some(aggregator) = aggregators.get_mut(&user_id) {
                    // Convert ProcessedItem back to InputItem for aggregator
                    // (In a full implementation, you might extend aggregators to handle ProcessedItem)
                    if let Err(e) = aggregator.add_item(processed_item.item).await {
                        warn!("Failed to add item to aggregator for user {}: {}", user_id, e);
                    }
                } else {
                    debug!("No aggregator found for user {}", user_id);
                }
            }
        });
    }
}

impl Default for IngestionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline builder for easier configuration
pub struct PipelineBuilder {
    pipeline: IngestionPipeline,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: IngestionPipeline::new(),
        }
    }
    
    pub fn add_source(mut self, source: Box<dyn PullFeed>) -> Self {
        self.pipeline.add_source(source);
        self
    }
    
    pub fn add_processing_stage(mut self, stage: Box<dyn ProcessingStage>) -> Self {
        self.pipeline.add_processing_stage(stage);
        self
    }
    
    pub async fn add_user_aggregator(
        self,
        user_id: String,
        aggregator_type: &str,
        config: Option<crate::traits::AggregatorConfig>,
    ) -> Result<Self> {
        self.pipeline.create_user_aggregator(user_id, aggregator_type, config).await?;
        Ok(self)
    }
    
    pub async fn add_user_preferences(
        self,
        user_id: String,
        preferences: DigestPreferences,
        memory: DigestModelMemory,
    ) -> Result<Self> {
        self.pipeline.set_user_preferences(user_id, preferences, memory).await?;
        Ok(self)
    }
    
    pub fn build(self) -> IngestionPipeline {
        self.pipeline
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}