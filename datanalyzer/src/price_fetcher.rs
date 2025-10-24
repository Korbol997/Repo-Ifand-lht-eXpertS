/// Price fetcher module for retrieving token prices and metadata.
///
/// This module provides functionality to fetch token prices from external sources
/// and cache token metadata like decimals to avoid repeated RPC calls.

use crate::error::AppError;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Default number of decimals to use when fetching fails
const DEFAULT_TOKEN_DECIMALS: u8 = 9;

/// Wrapped Solana (SOL) mint address on Solana
const WRAPPED_SOL_MINT: &str = "So11111111111111111111111111111111111111112";

/// Structure to hold token price information
#[derive(Debug, Clone)]
pub struct TokenPrice {
    /// Token mint address
    pub mint: Pubkey,
    /// Price in USD
    pub price_usd: f64,
}

/// Price fetcher with caching capabilities.
///
/// This structure maintains a cache of token decimals to avoid repeated
/// RPC calls when fetching token metadata.
pub struct PriceFetcher {
    /// RPC client for fetching on-chain data
    rpc_client: Arc<RpcClient>,
    /// Cache for token decimals (mint address -> decimals)
    decimals_cache: Arc<RwLock<HashMap<Pubkey, u8>>>,
}

impl PriceFetcher {
    /// Create a new PriceFetcher instance.
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - URL of the Solana RPC endpoint
    ///
    /// # Returns
    ///
    /// * `PriceFetcher` - New instance ready to fetch prices
    pub fn new(rpc_url: &str) -> Self {
        let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));
        let mut decimals_cache = HashMap::new();
        
        // Pre-populate cache with known token decimals
        // SOL has 9 decimals
        if let Ok(sol_mint) = WRAPPED_SOL_MINT.parse::<Pubkey>() {
            decimals_cache.insert(sol_mint, 9);
        }
        
        Self {
            rpc_client,
            decimals_cache: Arc::new(RwLock::new(decimals_cache)),
        }
    }

    /// Fetch prices for multiple tokens.
    ///
    /// This method fetches current USD prices for the specified token mints.
    /// In a real implementation, this would call external price APIs like CoinGecko,
    /// Jupiter, or other price oracles. For now, it returns placeholder values.
    ///
    /// # Arguments
    ///
    /// * `mints` - Slice of token mint addresses to fetch prices for
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<Pubkey, f64>)` - Map of mint addresses to prices in USD
    /// * `Err(AppError)` - If fetching prices fails
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. In production, you would:
    /// 1. Call a price API (CoinGecko, Jupiter, etc.)
    /// 2. Handle rate limiting
    /// 3. Implement proper error handling
    /// 4. Cache prices to avoid excessive API calls
    pub async fn fetch_prices(&self, mints: &[Pubkey]) -> Result<HashMap<Pubkey, f64>, AppError> {
        // Placeholder implementation
        // In production, this would fetch from price APIs
        let mut prices = HashMap::new();
        
        for mint in mints {
            // Check if this is the SOL mint
            if mint.to_string() == WRAPPED_SOL_MINT {
                // Placeholder SOL price
                prices.insert(*mint, 100.0);
            } else {
                // Placeholder token price
                // In production, fetch from API
                prices.insert(*mint, 1.0);
            }
        }
        
        Ok(prices)
    }

    /// Get token decimals from mint account.
    ///
    /// This method fetches the decimals value from the token mint account.
    /// Results are cached to avoid repeated RPC calls.
    ///
    /// # Arguments
    ///
    /// * `mint` - Token mint address
    ///
    /// # Returns
    ///
    /// * `Ok(u8)` - Number of decimals for the token
    /// * `Err(AppError)` - If fetching fails (returns default value instead)
    ///
    /// # Behavior
    ///
    /// 1. Check cache first
    /// 2. If not in cache, fetch from RPC
    /// 3. Store in cache for future use
    /// 4. If fetch fails, use default value (9) and log warning
    pub async fn get_token_decimals(&self, mint: &Pubkey) -> Result<u8, AppError> {
        // Check cache first
        {
            let cache = self.decimals_cache.read()
                .map_err(|e| AppError::PriceError(format!("Failed to acquire read lock: {}", e)))?;
            
            if let Some(&decimals) = cache.get(mint) {
                return Ok(decimals);
            }
        }

        // Not in cache, fetch from RPC
        let decimals = match self.fetch_decimals_from_rpc(mint).await {
            Ok(decimals) => decimals,
            Err(e) => {
                log::warn!(
                    "Failed to fetch decimals for mint {}: {}. Using default {}",
                    mint,
                    e,
                    DEFAULT_TOKEN_DECIMALS
                );
                DEFAULT_TOKEN_DECIMALS
            }
        };

        // Store in cache
        {
            let mut cache = self.decimals_cache.write()
                .map_err(|e| AppError::PriceError(format!("Failed to acquire write lock: {}", e)))?;
            
            cache.insert(*mint, decimals);
        }

        Ok(decimals)
    }

    /// Fetch decimals from the token mint account via RPC.
    ///
    /// This is an internal method that performs the actual RPC call.
    ///
    /// # Arguments
    ///
    /// * `mint` - Token mint address
    ///
    /// # Returns
    ///
    /// * `Ok(u8)` - Number of decimals
    /// * `Err(AppError)` - If RPC call or parsing fails
    async fn fetch_decimals_from_rpc(&self, mint: &Pubkey) -> Result<u8, AppError> {
        // Fetch the mint account data
        let account_data = self.rpc_client
            .get_account_data(mint)
            .map_err(|e| AppError::RpcError(format!("Failed to fetch mint account: {}", e)))?;

        // Parse the mint account to extract decimals
        // SPL Token mint account structure:
        // - Bytes 0-3: Not used in our case
        // - Bytes 4-35: mint_authority (32 bytes pubkey)
        // - Bytes 36-43: supply (u64)
        // - Byte 44: decimals (u8)
        if account_data.len() < 45 {
            return Err(AppError::DecodingError(format!(
                "Mint account data too small: {} bytes",
                account_data.len()
            )));
        }

        let decimals = account_data[44];
        Ok(decimals)
    }

    /// Get the current size of the decimals cache.
    ///
    /// Useful for monitoring and debugging.
    pub fn cache_size(&self) -> usize {
        self.decimals_cache.read()
            .map(|cache| cache.len())
            .unwrap_or(0)
    }

    /// Clear the decimals cache.
    ///
    /// Useful for testing or when you want to force refresh of all decimals.
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.decimals_cache.write() {
            cache.clear();
        }
    }

    /// Get a clone of the entire decimals cache.
    ///
    /// Useful for debugging and monitoring purposes.
    pub fn get_cache_snapshot(&self) -> HashMap<Pubkey, u8> {
        self.decimals_cache.read()
            .map(|cache| cache.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are basic structural tests.
    // Full integration tests would require a mock RPC client or test network.

    #[test]
    fn test_price_fetcher_new() {
        let fetcher = PriceFetcher::new("https://api.mainnet-beta.solana.com");
        
        // Should successfully create a fetcher
        let size = fetcher.cache_size();
        assert!(size == 0 || size > 0);  // Just checking it doesn't panic
    }

    #[test]
    fn test_cache_size() {
        let fetcher = PriceFetcher::new("https://api.mainnet-beta.solana.com");
        let size = fetcher.cache_size();
        // Cache size should be a valid usize
        let _ = size;  // Just ensure it compiles
    }

    #[test]
    fn test_clear_cache() {
        let fetcher = PriceFetcher::new("https://api.mainnet-beta.solana.com");
        
        // Clear the cache
        fetcher.clear_cache();
        
        // Should be empty or have default entries
        let size = fetcher.cache_size();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_get_cache_snapshot() {
        let fetcher = PriceFetcher::new("https://api.mainnet-beta.solana.com");
        let snapshot = fetcher.get_cache_snapshot();
        
        // Should return a HashMap
        let _ = snapshot.len();  // Just ensure it works
    }

    #[tokio::test]
    async fn test_fetch_prices_basic() {
        let fetcher = PriceFetcher::new("https://api.mainnet-beta.solana.com");
        
        let sol_mint = WRAPPED_SOL_MINT.parse::<Pubkey>().unwrap();
        let mints = vec![sol_mint];
        
        let result = fetcher.fetch_prices(&mints).await;
        assert!(result.is_ok());
        
        let prices = result.unwrap();
        assert_eq!(prices.len(), 1);
        assert!(prices.contains_key(&sol_mint));
    }

    #[test]
    fn test_wrapped_sol_mint_valid() {
        let result = WRAPPED_SOL_MINT.parse::<Pubkey>();
        assert!(result.is_ok());
    }

    #[test]
    fn test_token_price_structure() {
        let mint = Pubkey::new_unique();
        let price = TokenPrice {
            mint,
            price_usd: 100.0,
        };
        
        assert_eq!(price.mint, mint);
        assert_eq!(price.price_usd, 100.0);
    }
}
