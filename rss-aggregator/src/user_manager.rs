use crate::traits::{Aggregator, AggregatorConfig};
use crate::aggregators::TimeBucketAggregator;
use crate::types::{Result, AggregatorError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

/// Manages user-specific aggregators and their configurations
pub struct UserAggregatorManager {
    aggregators: Arc<RwLock<HashMap<String, Box<dyn Aggregator>>>>,
    default_configs: HashMap<String, AggregatorConfig>,
}

impl UserAggregatorManager {
    pub fn new() -> Self {
        let mut default_configs = HashMap::new();
        
        // Default daily digest configuration
        let mut daily_config = AggregatorConfig::default();
        daily_config.parameters.insert("bucket_duration_hours".to_string(), "24".to_string());
        daily_config.parameters.insert("max_items_per_bucket".to_string(), "50".to_string());
        default_configs.insert("daily".to_string(), daily_config);
        
        // Default hourly digest configuration
        let mut hourly_config = AggregatorConfig::default();
        hourly_config.parameters.insert("bucket_duration_hours".to_string(), "1".to_string());
        hourly_config.parameters.insert("max_items_per_bucket".to_string(), "20".to_string());
        default_configs.insert("hourly".to_string(), hourly_config);
        
        // Default weekly digest configuration
        let mut weekly_config = AggregatorConfig::default();
        weekly_config.parameters.insert("bucket_duration_hours".to_string(), "168".to_string());
        weekly_config.parameters.insert("max_items_per_bucket".to_string(), "100".to_string());
        default_configs.insert("weekly".to_string(), weekly_config);
        
        Self {
            aggregators: Arc::new(RwLock::new(HashMap::new())),
            default_configs,
        }
    }
    
    /// Create and register a new aggregator for a user
    pub async fn create_user_aggregator(
        &self,
        user_id: String,
        aggregator_type: &str,
        custom_config: Option<AggregatorConfig>,
    ) -> Result<()> {
        let aggregator: Box<dyn Aggregator> = match aggregator_type {
            "daily" => Box::new(TimeBucketAggregator::daily(user_id.clone())),
            "hourly" => Box::new(TimeBucketAggregator::hourly(user_id.clone())),
            "weekly" => Box::new(TimeBucketAggregator::weekly(user_id.clone())),
            "custom_time_bucket" => {
                // For custom time bucket, require duration in config
                let config = custom_config.as_ref().ok_or_else(|| {
                    AggregatorError::General("Custom time bucket requires configuration".to_string())
                })?;
                
                let duration_hours = config
                    .parameters
                    .get("bucket_duration_hours")
                    .and_then(|d| d.parse::<i64>().ok())
                    .ok_or_else(|| {
                        AggregatorError::General("bucket_duration_hours required for custom time bucket".to_string())
                    })?;
                
                Box::new(TimeBucketAggregator::new_with_duration(user_id.clone(), duration_hours))
            }
            _ => {
                return Err(AggregatorError::General(format!("Unknown aggregator type: {}", aggregator_type)));
            }
        };
        
        let mut aggregators = self.aggregators.write().await;
        
        // Apply configuration
        if let Some(aggregator_ref) = aggregators.get_mut(&user_id) {
            let config = custom_config.unwrap_or_else(|| {
                self.default_configs.get(aggregator_type).cloned().unwrap_or_default()
            });
            aggregator_ref.configure(config).await?;
        } else {
            // Insert new aggregator
            aggregators.insert(user_id.clone(), aggregator);
            
            // Apply configuration to the newly inserted aggregator
            if let Some(aggregator_ref) = aggregators.get_mut(&user_id) {
                let config = custom_config.unwrap_or_else(|| {
                    self.default_configs.get(aggregator_type).cloned().unwrap_or_default()
                });
                aggregator_ref.configure(config).await?;
            }
        }
        
        info!("Created {} aggregator for user {}", aggregator_type, user_id);
        Ok(())
    }
    
    /// Remove a user's aggregator
    pub async fn remove_user_aggregator(&self, user_id: &str) -> Result<bool> {
        let mut aggregators = self.aggregators.write().await;
        let removed = aggregators.remove(user_id).is_some();
        
        if removed {
            info!("Removed aggregator for user {}", user_id);
        } else {
            debug!("No aggregator found to remove for user {}", user_id);
        }
        
        Ok(removed)
    }
    
    /// Get a read-only reference to a user's aggregator
    pub async fn get_user_aggregator(&self, user_id: &str) -> Option<String> {
        let aggregators = self.aggregators.read().await;
        aggregators.get(user_id).map(|agg| agg.aggregator_type())
    }
    
    /// Update configuration for an existing user aggregator
    pub async fn configure_user_aggregator(
        &self,
        user_id: &str,
        config: AggregatorConfig,
    ) -> Result<()> {
        let mut aggregators = self.aggregators.write().await;
        
        if let Some(aggregator) = aggregators.get_mut(user_id) {
            aggregator.configure(config).await?;
            info!("Updated configuration for user {} aggregator", user_id);
            Ok(())
        } else {
            Err(AggregatorError::General(format!("No aggregator found for user {}", user_id)))
        }
    }
    
    /// Get list of all managed users
    pub async fn get_managed_users(&self) -> Vec<String> {
        let aggregators = self.aggregators.read().await;
        aggregators.keys().cloned().collect()
    }
    
    /// Get statistics about managed aggregators
    pub async fn get_manager_stats(&self) -> ManagerStats {
        let aggregators = self.aggregators.read().await;
        let total_users = aggregators.len();
        
        let mut aggregator_types = HashMap::new();
        for aggregator in aggregators.values() {
            let agg_type = aggregator.aggregator_type();
            *aggregator_types.entry(agg_type).or_insert(0) += 1;
        }
        
        ManagerStats {
            total_users,
            aggregator_types,
        }
    }
    
    /// Bulk create aggregators for multiple users
    pub async fn bulk_create_aggregators(
        &self,
        user_configs: Vec<(String, String, Option<AggregatorConfig>)>, // (user_id, aggregator_type, config)
    ) -> Result<BulkCreateResult> {
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        
        for (user_id, aggregator_type, config) in user_configs {
            match self.create_user_aggregator(user_id.clone(), &aggregator_type, config).await {
                Ok(()) => successful.push(user_id),
                Err(e) => {
                    warn!("Failed to create aggregator for user {}: {}", user_id, e);
                    failed.push((user_id, e.to_string()));
                }
            }
        }
        
        Ok(BulkCreateResult { successful, failed })
    }
    
    /// Get access to the internal aggregators map (for pipeline integration)
    pub fn get_aggregators_map(&self) -> Arc<RwLock<HashMap<String, Box<dyn Aggregator>>>> {
        self.aggregators.clone()
    }
    
    /// Check if a user has an aggregator
    pub async fn has_user_aggregator(&self, user_id: &str) -> bool {
        let aggregators = self.aggregators.read().await;
        aggregators.contains_key(user_id)
    }
}

/// Statistics about the aggregator manager
#[derive(Debug, Clone)]
pub struct ManagerStats {
    pub total_users: usize,
    pub aggregator_types: HashMap<String, usize>,
}

/// Result of bulk aggregator creation
#[derive(Debug)]
pub struct BulkCreateResult {
    pub successful: Vec<String>,
    pub failed: Vec<(String, String)>, // (user_id, error_message)
}

impl Default for UserAggregatorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating user aggregator configurations
pub struct UserAggregatorBuilder {
    user_id: String,
    aggregator_type: String,
    config: AggregatorConfig,
}

impl UserAggregatorBuilder {
    pub fn new(user_id: String, aggregator_type: String) -> Self {
        Self {
            user_id,
            aggregator_type,
            config: AggregatorConfig::default(),
        }
    }
    
    pub fn with_bucket_duration_hours(mut self, hours: i64) -> Self {
        self.config.parameters.insert("bucket_duration_hours".to_string(), hours.to_string());
        self
    }
    
    pub fn with_max_items(mut self, max_items: usize) -> Self {
        self.config.parameters.insert("max_items_per_bucket".to_string(), max_items.to_string());
        self
    }
    
    pub fn with_parameter(mut self, key: String, value: String) -> Self {
        self.config.parameters.insert(key, value);
        self
    }
    
    pub async fn create(self, manager: &UserAggregatorManager) -> Result<()> {
        manager.create_user_aggregator(self.user_id, &self.aggregator_type, Some(self.config)).await
    }
}

/// Convenience functions for common aggregator setups
impl UserAggregatorManager {
    /// Create a daily digest aggregator with custom item limit
    pub async fn create_daily_digest(&self, user_id: String, max_items: Option<usize>) -> Result<()> {
        let mut config = self.default_configs.get("daily").cloned().unwrap_or_default();
        if let Some(max) = max_items {
            config.parameters.insert("max_items_per_bucket".to_string(), max.to_string());
        }
        self.create_user_aggregator(user_id, "daily", Some(config)).await
    }
    
    /// Create a custom time bucket aggregator
    pub async fn create_custom_time_bucket(
        &self,
        user_id: String,
        duration_hours: i64,
        max_items: Option<usize>,
    ) -> Result<()> {
        let builder = UserAggregatorBuilder::new(user_id, "custom_time_bucket".to_string())
            .with_bucket_duration_hours(duration_hours);
        
        let builder = if let Some(max) = max_items {
            builder.with_max_items(max)
        } else {
            builder
        };
        
        builder.create(self).await
    }
}