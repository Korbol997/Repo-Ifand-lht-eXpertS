/// DEX decoder module for parsing liquidity pool account data.
/// 
/// This module provides traits and implementations for decoding various DEX
/// pool account structures on Solana. Each DEX has its own data layout,
/// and this module abstracts the decoding logic behind a common interface.

pub mod pumpfun;
pub mod raydium;

use crate::error::AppError;

/// Trait for decoding DEX pool account data.
///
/// This trait defines the interface for extracting reserve amounts and validating
/// account data for different DEX implementations (Pump.fun, Raydium, etc.).
///
/// # Contract
///
/// Implementations of this trait must:
/// - Parse account data according to the specific DEX's data layout
/// - Return reserves as (base_reserve, quote_reserve) or (token_reserve, sol_reserve)
/// - Validate account data before decoding to prevent panics
/// - Use little-endian byte order for deserializing u64 values (standard for Solana)
/// - Be thread-safe (Send + Sync) for use in async contexts
///
/// # Example
///
/// ```ignore
/// use crate::dex::{DexDecoder, pumpfun::PumpFunDecoder};
///
/// let decoder = PumpFunDecoder;
/// let account_data: &[u8] = /* ... fetched from Solana RPC ... */;
///
/// // Validate the account data first
/// decoder.validate_account(account_data)?;
///
/// // Decode the reserves
/// let (token_reserve, sol_reserve) = decoder.decode_reserves(account_data)?;
/// println!("Token Reserve: {}, SOL Reserve: {}", token_reserve, sol_reserve);
/// ```
pub trait DexDecoder: Send + Sync {
    /// Decode reserve amounts from account data.
    ///
    /// Extracts the reserve amounts from the raw account data. For most DEXs,
    /// this returns (base_reserve, quote_reserve), typically representing
    /// (token_amount, sol_amount).
    ///
    /// # Arguments
    ///
    /// * `account_data` - Raw bytes from the Solana account
    ///
    /// # Returns
    ///
    /// * `Ok((u64, u64))` - Tuple of (base_reserve, quote_reserve)
    /// * `Err(AppError)` - If decoding fails due to invalid data, incorrect size, etc.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Account data is too small to contain the required fields
    /// - Data is corrupted or cannot be properly deserialized
    /// - Reserved values are invalid (e.g., both reserves are zero)
    fn decode_reserves(&self, account_data: &[u8]) -> Result<(u64, u64), AppError>;

    /// Validate account data before decoding.
    ///
    /// Performs validation checks to ensure the account data is valid for this
    /// specific DEX. This should be called before `decode_reserves` to prevent
    /// errors and ensure data integrity.
    ///
    /// # Arguments
    ///
    /// * `account_data` - Raw bytes from the Solana account
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the account data is valid
    /// * `Err(AppError)` - If validation fails with detailed error message
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Account size is incorrect
    /// - Discriminator/magic bytes don't match expected values
    /// - Critical fields contain invalid values
    fn validate_account(&self, account_data: &[u8]) -> Result<(), AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that the trait is object-safe (can be used as Box<dyn DexDecoder>)
    #[test]
    fn test_trait_is_object_safe() {
        fn _assert_object_safe(_decoder: &dyn DexDecoder) {}
    }
}
