# Repo-Ifand-lht-eXpertS

Solana DEX Pool Monitor - A tool for monitoring liquidity pools on various Solana DEXs.

## Features

- ✅ DEX Decoder abstraction for parsing pool account data
- ✅ Pump.fun bonding curve decoder (fully implemented)
- ✅ Raydium AMM V4 decoder (structure complete, vault extraction)
- Configuration-based pool monitoring
- CSV export for pool snapshots

## DEX Decoder Implementation

### Pump.fun Decoder
Fully implemented decoder for Pump.fun bonding curve accounts. Extracts real token and SOL reserves directly from the 256-byte account data.

See [DEX_DECODER_IMPLEMENTATION.md](datanalyzer/DEX_DECODER_IMPLEMENTATION.md) for detailed documentation.

### Raydium AMM V4 Decoder
Complete implementation of Raydium AMM V4 pool decoder with full AmmInfo structure (752 bytes). Extracts vault pubkeys for fetching actual reserves.

See [RAYDIUM_IMPLEMENTATION.md](datanalyzer/RAYDIUM_IMPLEMENTATION.md) for comprehensive guide.

### Quick Start

```rust
use datanalyzer::dex::{DexDecoder, pumpfun::PumpFunDecoder};
use datanalyzer::dex::raydium::RaydiumDecoder;

// Pump.fun - get reserves directly
let pumpfun_decoder = PumpFunDecoder;
let (token_reserve, sol_reserve) = pumpfun_decoder.decode_reserves(&account_data)?;

// Raydium - get vault pubkeys
let raydium_decoder = RaydiumDecoder;
let vault_info = raydium_decoder.get_vault_info(&account_data)?;
println!("Coin vault: {}", vault_info.coin_vault);
println!("PC vault: {}", vault_info.pc_vault);
```

## Building

```bash
cd datanalyzer
cargo build
```

## Testing

```bash
cd datanalyzer
cargo test
```

All tests currently passing (51/51).

## Implementation Status

### ✅ Completed Tasks

#### Task 4.1 - Raydium AmmInfo Structure
- Complete AmmInfo structure definition (752 bytes)
- Fees structure (64 bytes)
- StateData structure (144 bytes)
- VaultInfo helper structure
- Full compatibility with official Raydium implementation

#### Task 4.2 - RaydiumDecoder Implementation
- Bytemuck-based deserialization
- Account validation (size, status, vault pubkeys)
- Vault pubkey extraction
- Helper methods for convenience

#### Task 4.3 - Vault Account Handling
- Research complete - vault accounts are separate SPL token accounts
- Documentation of architecture
- VaultInfo helper for pubkey management
- Guide for future async RPC implementation

#### Task 4.4 - Tests
- 16 Raydium-specific tests
- Structure size validation
- Deserialization tests
- Vault extraction tests
- Realistic pool data tests
- All tests passing (51 total)

## Architecture

### Pump.fun vs Raydium

**Pump.fun:**
- Stores reserves directly in pool account
- Single account read for all data
- Simple synchronous decoding

**Raydium:**
- Stores vault pubkeys in pool account
- Requires separate vault account reads
- Two-step process: decode pool → fetch vaults

## Documentation

- [DEX Decoder Architecture](datanalyzer/DEX_DECODER_IMPLEMENTATION.md)
- [Raydium Implementation Guide](datanalyzer/RAYDIUM_IMPLEMENTATION.md)
- Inline code documentation

## Security

✅ No vulnerabilities detected (CodeQL scan passed)