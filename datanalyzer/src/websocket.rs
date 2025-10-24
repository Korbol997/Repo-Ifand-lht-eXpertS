use crate::config::PoolConfig;
use crate::error::AppError;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_client::rpc_response::{Response, RpcKeyedAccount};
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Callback function type for processing account updates
pub type AccountUpdateCallback = Arc<dyn Fn(Pubkey, Vec<u8>, u64) + Send + Sync>;

/// WebSocket manager for subscribing to Solana account updates
pub struct WebSocketManager {
    ws_url: String,
    client: Option<Arc<PubsubClient>>,
    subscriptions: Arc<Mutex<HashMap<Pubkey, u64>>>,
    last_snapshot_time: Arc<Mutex<HashMap<Pubkey, Instant>>>,
    snapshot_interval_ms: u64,
    skipped_notifications: Arc<Mutex<HashMap<Pubkey, u64>>>,
    next_subscription_id: Arc<Mutex<u64>>,
}

impl WebSocketManager {
    /// Create a new WebSocketManager instance
    ///
    /// # Arguments
    ///
    /// * `ws_url` - WebSocket URL for the Solana RPC endpoint
    /// * `snapshot_interval_ms` - Minimum interval between snapshots in milliseconds (0 to disable throttling)
    pub fn new(ws_url: String, snapshot_interval_ms: u64) -> Self {
        Self {
            ws_url,
            client: None,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            last_snapshot_time: Arc::new(Mutex::new(HashMap::new())),
            snapshot_interval_ms,
            skipped_notifications: Arc::new(Mutex::new(HashMap::new())),
            next_subscription_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Connect to the WebSocket endpoint with timeout
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If connection succeeded
    /// * `Err(AppError)` - If connection failed with detailed error message
    pub async fn connect(&mut self) -> Result<(), AppError> {
        let ws_url = self.ws_url.clone();
        
        log::info!("Attempting to connect to WebSocket at: {}", ws_url);
        
        // Attempt connection with 10 second timeout
        match tokio::time::timeout(Duration::from_secs(10), PubsubClient::new(&ws_url)).await {
            Ok(Ok(client)) => {
                self.client = Some(Arc::new(client));
                let timestamp = chrono::Utc::now().to_rfc3339();
                log::info!("Successfully connected to WebSocket at {}", timestamp);
                Ok(())
            }
            Ok(Err(e)) => {
                Err(AppError::RpcError(format!(
                    "Failed to create PubsubClient: {}. Please check if the URL is valid and the endpoint is accessible.",
                    e
                )))
            }
            Err(_) => {
                Err(AppError::RpcError(
                    "Connection timeout after 10 seconds. Please check your network connection and endpoint availability.".to_string()
                ))
            }
        }
    }

    /// Subscribe to a single pool's account updates
    ///
    /// # Arguments
    ///
    /// * `pool_address` - The Pubkey of the pool to subscribe to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If subscription succeeded
    /// * `Err(AppError)` - If subscription failed
    pub async fn subscribe_pool(&self, pool_address: Pubkey) -> Result<(), AppError> {
        let client = self.client.as_ref()
            .ok_or_else(|| AppError::RpcError("Not connected. Call connect() first.".to_string()))?;
        
        // Subscribe with commitment config
        let config = RpcAccountInfoConfig {
            encoding: None,
            commitment: Some(CommitmentConfig::confirmed()),
            data_slice: None,
            min_context_slot: None,
        };
        
        match client.account_subscribe(&pool_address, Some(config)).await {
            Ok(_subscription_stream) => {
                // Generate and store a subscription ID
                let mut next_id = self.next_subscription_id.lock().await;
                let subscription_id = *next_id;
                *next_id += 1;
                
                let mut subscriptions = self.subscriptions.lock().await;
                subscriptions.insert(pool_address, subscription_id);
                log::info!("Subscribed to pool {} with subscription ID: {}", pool_address, subscription_id);
                
                // Note: In a real implementation, we would spawn a task to listen to the stream
                // For now, we're just tracking that the subscription was created
                
                Ok(())
            }
            Err(e) => {
                Err(AppError::RpcError(format!(
                    "Failed to subscribe to pool {}: {}",
                    pool_address, e
                )))
            }
        }
    }

    /// Subscribe to all pools in the configuration
    ///
    /// # Arguments
    ///
    /// * `pools` - Slice of PoolConfig instances to subscribe to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all subscriptions succeeded
    /// * `Err(AppError)` - If any subscription failed (returns first error)
    pub async fn subscribe_all_pools(&self, pools: &[PoolConfig]) -> Result<(), AppError> {
        for pool in pools {
            self.subscribe_pool(*pool.pool_address()).await?;
        }
        log::info!("Successfully subscribed to {} pools", pools.len());
        Ok(())
    }

    /// Check if a notification should be processed based on throttling settings
    ///
    /// # Arguments
    ///
    /// * `pool_address` - The pool address to check
    ///
    /// # Returns
    ///
    /// * `true` - If notification should be processed
    /// * `false` - If notification should be skipped due to throttling
    async fn should_process_notification(&self, pool_address: &Pubkey) -> bool {
        // If throttling is disabled (0), always process
        if self.snapshot_interval_ms == 0 {
            return true;
        }

        let mut last_times = self.last_snapshot_time.lock().await;
        let now = Instant::now();

        if let Some(last_time) = last_times.get(pool_address) {
            let elapsed = now.duration_since(*last_time);
            let threshold = Duration::from_millis(self.snapshot_interval_ms);

            if elapsed < threshold {
                // Increment skipped counter
                let mut skipped = self.skipped_notifications.lock().await;
                *skipped.entry(*pool_address).or_insert(0) += 1;
                return false;
            }
        }

        // Update last snapshot time
        last_times.insert(*pool_address, now);
        true
    }

    /// Log statistics about skipped notifications
    pub async fn log_skipped_stats(&self) {
        let skipped = self.skipped_notifications.lock().await;
        if !skipped.is_empty() {
            log::info!("=== Throttling Statistics ===");
            for (pool, count) in skipped.iter() {
                log::info!("Pool {}: {} notifications skipped", pool, count);
            }
        }
    }

    /// Reset skipped notification counters
    pub async fn reset_skipped_stats(&self) {
        let mut skipped = self.skipped_notifications.lock().await;
        skipped.clear();
    }

    /// Get the number of active subscriptions
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.lock().await.len()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    /// Get WebSocket URL
    pub fn get_ws_url(&self) -> &str {
        &self.ws_url
    }

    /// Get snapshot interval in milliseconds
    pub fn get_snapshot_interval_ms(&self) -> u64 {
        self.snapshot_interval_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_websocket_manager_new() {
        let ws_url = "wss://api.mainnet-beta.solana.com".to_string();
        let manager = WebSocketManager::new(ws_url.clone(), 1000);

        assert_eq!(manager.get_ws_url(), ws_url);
        assert_eq!(manager.get_snapshot_interval_ms(), 1000);
        assert!(!manager.is_connected());
    }

    #[test]
    fn test_websocket_manager_new_no_throttling() {
        let ws_url = "wss://api.mainnet-beta.solana.com".to_string();
        let manager = WebSocketManager::new(ws_url, 0);

        assert_eq!(manager.get_snapshot_interval_ms(), 0);
    }

    #[tokio::test]
    async fn test_subscription_count_empty() {
        let ws_url = "wss://api.mainnet-beta.solana.com".to_string();
        let manager = WebSocketManager::new(ws_url, 1000);

        assert_eq!(manager.subscription_count().await, 0);
    }

    #[tokio::test]
    async fn test_should_process_notification_no_throttling() {
        let ws_url = "wss://api.mainnet-beta.solana.com".to_string();
        let manager = WebSocketManager::new(ws_url, 0);

        let pool = Pubkey::new_unique();
        
        // Should always process when throttling is disabled
        assert!(manager.should_process_notification(&pool).await);
        assert!(manager.should_process_notification(&pool).await);
        assert!(manager.should_process_notification(&pool).await);
    }

    #[tokio::test]
    async fn test_should_process_notification_with_throttling() {
        let ws_url = "wss://api.mainnet-beta.solana.com".to_string();
        let manager = WebSocketManager::new(ws_url, 100);

        let pool = Pubkey::new_unique();
        
        // First notification should be processed
        assert!(manager.should_process_notification(&pool).await);
        
        // Immediate second notification should be skipped
        assert!(!manager.should_process_notification(&pool).await);
        
        // Wait for throttling interval to pass
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should be processed again
        assert!(manager.should_process_notification(&pool).await);
    }

    #[tokio::test]
    async fn test_skipped_stats() {
        let ws_url = "wss://api.mainnet-beta.solana.com".to_string();
        let manager = WebSocketManager::new(ws_url, 100);

        let pool = Pubkey::new_unique();
        
        // Process first
        assert!(manager.should_process_notification(&pool).await);
        
        // Skip next 5
        for _ in 0..5 {
            assert!(!manager.should_process_notification(&pool).await);
        }
        
        // Check stats were recorded
        let skipped = manager.skipped_notifications.lock().await;
        assert_eq!(skipped.get(&pool), Some(&5));
    }

    #[tokio::test]
    async fn test_reset_skipped_stats() {
        let ws_url = "wss://api.mainnet-beta.solana.com".to_string();
        let manager = WebSocketManager::new(ws_url, 100);

        let pool = Pubkey::new_unique();
        
        // Generate some skipped notifications
        manager.should_process_notification(&pool).await;
        manager.should_process_notification(&pool).await;
        
        // Reset stats
        manager.reset_skipped_stats().await;
        
        let skipped = manager.skipped_notifications.lock().await;
        assert_eq!(skipped.len(), 0);
    }

    #[tokio::test]
    async fn test_connect_invalid_url() {
        let mut manager = WebSocketManager::new("invalid_url".to_string(), 1000);
        
        let result = manager.connect().await;
        assert!(result.is_err());
    }
}
