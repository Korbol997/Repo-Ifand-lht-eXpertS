use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DexType {
    PumpFun,
    Raydium,
}

impl FromStr for DexType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pumpfun" | "pump_fun" => Ok(DexType::PumpFun),
            "raydium" => Ok(DexType::Raydium),
            _ => Err(AppError::ConfigError(format!("Unknown DEX type: {}", s))),
        }
    }
}

impl fmt::Display for DexType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DexType::PumpFun => write!(f, "PumpFun"),
            DexType::Raydium => write!(f, "Raydium"),
        }
    }
}

impl DexType {
    pub fn get_account_size(&self) -> usize {
        match self {
            DexType::PumpFun => 256,
            DexType::Raydium => 752,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSnapshot {
    pub pool_address: String,
    pub token_mint: String,
    pub dex_type: DexType,
    pub reserve_base: u64,
    pub reserve_quote: u64,
    pub timestamp: i64,
    pub price: f64,
}

impl PoolSnapshot {
    pub fn new(
        pool_address: String,
        token_mint: String,
        dex_type: DexType,
        reserve_base: u64,
        reserve_quote: u64,
        timestamp: i64,
        price: f64,
    ) -> Result<Self, AppError> {
        if reserve_base == 0 || reserve_quote == 0 {
            return Err(AppError::ConfigError(
                "Reserves cannot be zero".to_string(),
            ));
        }

        Ok(PoolSnapshot {
            pool_address,
            token_mint,
            dex_type,
            reserve_base,
            reserve_quote,
            timestamp,
            price,
        })
    }

    pub fn to_csv_row(&self) -> Vec<String> {
        vec![
            self.pool_address.clone(),
            self.token_mint.clone(),
            self.dex_type.to_string(),
            self.reserve_base.to_string(),
            self.reserve_quote.to_string(),
            self.timestamp.to_string(),
            self.price.to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dex_type_from_str() {
        assert_eq!("pumpfun".parse::<DexType>().unwrap(), DexType::PumpFun);
        assert_eq!("PumpFun".parse::<DexType>().unwrap(), DexType::PumpFun);
        assert_eq!("pump_fun".parse::<DexType>().unwrap(), DexType::PumpFun);
        assert_eq!("raydium".parse::<DexType>().unwrap(), DexType::Raydium);
        assert_eq!("Raydium".parse::<DexType>().unwrap(), DexType::Raydium);
        
        assert!("unknown".parse::<DexType>().is_err());
    }

    #[test]
    fn test_dex_type_display() {
        assert_eq!(format!("{}", DexType::PumpFun), "PumpFun");
        assert_eq!(format!("{}", DexType::Raydium), "Raydium");
    }

    #[test]
    fn test_dex_type_account_size() {
        assert_eq!(DexType::PumpFun.get_account_size(), 256);
        assert_eq!(DexType::Raydium.get_account_size(), 752);
    }

    #[test]
    fn test_pool_snapshot_validation() {
        let result = PoolSnapshot::new(
            "pool123".to_string(),
            "token456".to_string(),
            DexType::Raydium,
            0,
            1000,
            1234567890,
            1.5,
        );
        assert!(result.is_err());

        let result = PoolSnapshot::new(
            "pool123".to_string(),
            "token456".to_string(),
            DexType::Raydium,
            1000,
            0,
            1234567890,
            1.5,
        );
        assert!(result.is_err());

        let result = PoolSnapshot::new(
            "pool123".to_string(),
            "token456".to_string(),
            DexType::Raydium,
            1000,
            2000,
            1234567890,
            1.5,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_pool_snapshot_to_csv_row() {
        let snapshot = PoolSnapshot::new(
            "pool123".to_string(),
            "token456".to_string(),
            DexType::PumpFun,
            1000,
            2000,
            1234567890,
            0.5,
        )
        .unwrap();

        let csv_row = snapshot.to_csv_row();
        assert_eq!(csv_row.len(), 7);
        assert_eq!(csv_row[0], "pool123");
        assert_eq!(csv_row[1], "token456");
        assert_eq!(csv_row[2], "PumpFun");
        assert_eq!(csv_row[3], "1000");
        assert_eq!(csv_row[4], "2000");
        assert_eq!(csv_row[5], "1234567890");
        assert_eq!(csv_row[6], "0.5");
    }
}
