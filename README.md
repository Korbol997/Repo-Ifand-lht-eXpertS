# Repo-Ifand-lht-eXpertS

Solana DEX Pool Monitor - A tool for monitoring liquidity pools on various Solana DEXs.

## Features

- ✅ DEX Decoder abstraction for parsing pool account data
- ✅ Pump.fun bonding curve decoder (fully implemented)
- 🚧 Raydium AMM decoder (placeholder)
- Configuration-based pool monitoring
- CSV export for pool snapshots

## DEX Decoder Implementation

See [DEX_DECODER_IMPLEMENTATION.md](datanalyzer/DEX_DECODER_IMPLEMENTATION.md) for detailed documentation on the DEX decoder architecture and usage.

### Quick Start

```rust
use datanalyzer::dex::{DexDecoder, pumpfun::PumpFunDecoder};

let decoder = PumpFunDecoder;
let (token_reserve, sol_reserve) = decoder.decode_reserves(&account_data)?;
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

All tests currently passing (38/38).