# Decoder Factory and Registry Implementation

## Overview

This document describes the implementation of the decoder factory pattern and registry system for managing DEX pool decoders in the Datanalyzer project.

## Features Implemented

### Task 5: Decoder Factory

**Function**: `create_decoder(dex_type: DexType) -> Box<dyn DexDecoder>`

A factory function that creates decoder instances based on DEX type using pattern matching.

**Features:**
- Thread-safe (decoders are Send + Sync)
- Returns boxed trait objects for polymorphism
- Simple pattern matching on DexType enum

**Usage:**
```rust
use datanalyzer::dex::create_decoder;
use datanalyzer::models::DexType;

let decoder = create_decoder(DexType::PumpFun);
let (token_reserve, sol_reserve) = decoder.decode_reserves(&account_data)?;
```

### Task 5.1: Decoder Registry

**Structure**: `DecoderRegistry`

A thread-safe registry for managing multiple pool decoders using a HashMap.

**Features:**
- `HashMap<Pubkey, Box<dyn DexDecoder>>` for storing decoders
- Thread-safe access using `Arc<RwLock<T>>`
- Support for multiple concurrent readers
- Optimized for read-heavy workloads

**Methods:**
- `register_pool(pool_config: PoolConfig)` - Register a pool and create its decoder immediately
- `get_decoder(pool_address: &Pubkey)` - Check if a decoder exists for a pool
- `decode_pool_data(pool_address, account_data)` - Facade method for decoding pool data

**Usage:**
```rust
use datanalyzer::dex::DecoderRegistry;

let registry = DecoderRegistry::new();

// Register pools
registry.register_pool(pool_config1)?;
registry.register_pool(pool_config2)?;

// Decode pool data
let (base, quote) = registry.decode_pool_data(&pool_address, &account_data)?;
```

### Task 5.2: Lazy Initialization

**Features:**
- Decoders created only when first needed
- Double-check locking pattern for thread safety
- Separate pool type registration from decoder creation
- Optimal memory usage for large numbers of pools

**Methods:**
- `register_pool_type(pool_config)` - Register pool type without creating decoder
- `get_or_create_decoder(pool_address)` - Create decoder on first access
- `decode_pool_data_lazy(pool_address, account_data)` - Decode with lazy initialization

**Double-Check Locking Pattern:**
1. First check with read lock (fast path)
2. If not found, acquire write lock
3. Check again (another thread might have created it)
4. Create decoder only if still not present

**Usage:**
```rust
let registry = DecoderRegistry::new();

// Register pool type only (no decoder created yet)
registry.register_pool_type(pool_config)?;

// Decoder created on first use
let (base, quote) = registry.decode_pool_data_lazy(&pool_address, &account_data)?;
```

### Task 5.3: Metrics and Monitoring

**Structure**: `DecoderStats`

Tracks success and error statistics for each decoder.

**Fields:**
- `success_count: u64` - Number of successful decodings
- `error_count: u64` - Number of failed decodings
- `first_success_time: Option<u64>` - Timestamp of first success
- `last_success_time: Option<u64>` - Timestamp of last success
- `last_error_time: Option<u64>` - Timestamp of last error

**Methods:**
- `get_stats(pool_address)` - Get statistics for a specific pool
- `get_all_stats()` - Get statistics for all pools
- `log_stats(force: bool)` - Log statistics (auto-throttled to 5-minute intervals)
- `reset_stats()` - Reset all statistics
- `reset_pool_stats(pool_address)` - Reset statistics for one pool

**Automatic Tracking:**
All decode operations automatically update statistics:
- Successful decodes increment `success_count`
- Failed decodes increment `error_count`
- Timestamps are automatically recorded

**Usage:**
```rust
// Statistics are tracked automatically
registry.decode_pool_data(&pool_address, &account_data)?;

// View statistics
if let Some(stats) = registry.get_stats(&pool_address) {
    println!("Success rate: {:.2}%", stats.success_rate());
    println!("Total attempts: {}", stats.total_attempts());
}

// Log all statistics (throttled to 5-minute intervals)
registry.log_stats(false);

// Force immediate logging
registry.log_stats(true);
```

## Thread Safety

All components are designed for safe concurrent access:

### RwLock Usage
- **Read Lock**: Used for decoder lookups and statistics reads
- **Write Lock**: Used only for creating new decoders and updating statistics
- **Optimization**: Multiple readers can access simultaneously without blocking

### Arc for Shared Ownership
- Allows registry to be shared across threads
- Safe reference counting for thread-safe access

### Lock Ordering
- Statistics updates use separate lock to avoid deadlocks
- Read locks released before acquiring write locks

## Performance Considerations

### Lazy Initialization Benefits
1. **Memory**: Decoders only created when needed
2. **Startup**: Faster application startup with many pools
3. **Flexibility**: Support for dynamic pool addition

### Read-Heavy Optimization
1. **RwLock**: Allows multiple concurrent readers
2. **Double-Check Locking**: Minimizes write lock acquisition
3. **Lock Granularity**: Separate locks for decoders and statistics

### Statistics Overhead
- Minimal: Simple counter increments and timestamp recording
- No blocking: Statistics updates don't block decode operations
- Throttling: Log operations throttled to prevent spam

## Example Usage

See `examples/decoder_registry_demo.rs` for a complete working example demonstrating:
1. Factory pattern usage
2. Registry with immediate initialization
3. Registry with lazy initialization
4. Statistics tracking and logging

Run the example:
```bash
cargo run --example decoder_registry_demo
```

## Testing

Comprehensive test suite with 75 tests covering:
- Factory function for both PumpFun and Raydium
- Registry registration and retrieval
- Lazy initialization patterns
- Statistics tracking and accuracy
- Thread safety guarantees
- Error handling

Run tests:
```bash
cargo test
```

## API Reference

### create_decoder
```rust
pub fn create_decoder(dex_type: DexType) -> Box<dyn DexDecoder>
```

### DecoderRegistry
```rust
impl DecoderRegistry {
    pub fn new() -> Self
    pub fn register_pool(&self, pool_config: PoolConfig) -> Result<(), AppError>
    pub fn register_pool_type(&self, pool_config: PoolConfig) -> Result<(), AppError>
    pub fn get_decoder(&self, pool_address: &Pubkey) -> Result<(), AppError>
    pub fn decode_pool_data(&self, pool_address: &Pubkey, account_data: &[u8]) -> Result<(u64, u64), AppError>
    pub fn decode_pool_data_lazy(&self, pool_address: &Pubkey, account_data: &[u8]) -> Result<(u64, u64), AppError>
    pub fn get_or_create_decoder(&self, pool_address: &Pubkey) -> Result<(), AppError>
    pub fn pool_count(&self) -> usize
    pub fn get_stats(&self, pool_address: &Pubkey) -> Option<DecoderStats>
    pub fn get_all_stats(&self) -> HashMap<Pubkey, DecoderStats>
    pub fn log_stats(&self, force: bool) -> bool
    pub fn reset_stats(&self)
    pub fn reset_pool_stats(&self, pool_address: &Pubkey)
}
```

### DecoderStats
```rust
impl DecoderStats {
    pub fn new() -> Self
    pub fn success_rate(&self) -> f64
    pub fn total_attempts(&self) -> u64
}
```

## Security

- CodeQL security scan: ✅ 0 vulnerabilities
- No unsafe code used
- All error cases properly handled
- Thread-safe by design

## Future Enhancements

Possible future improvements:
1. Persistent statistics (save to file/database)
2. Metrics export to Prometheus/StatsD
3. Configurable logging intervals
4. Performance histograms (decode time tracking)
5. Alerting on error rate thresholds
6. Circuit breaker pattern for failing decoders
