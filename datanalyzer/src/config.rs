use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::error::AppError;
use crate::models::DexType;

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pool_address: Pubkey,
    dex_type: DexType,
    token_mint: Pubkey,
}

impl PoolConfig {
    pub fn new(pool_address: Pubkey, dex_type: DexType, token_mint: Pubkey) -> Result<Self, AppError> {
        let config = PoolConfig {
            pool_address,
            dex_type,
            token_mint,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.pool_address == Pubkey::default() {
            return Err(AppError::ConfigError(
                "Pool address cannot be default/zero pubkey".to_string(),
            ));
        }
        
        if self.token_mint == Pubkey::default() {
            return Err(AppError::ConfigError(
                "Token mint cannot be default/zero pubkey".to_string(),
            ));
        }
        
        Ok(())
    }

    pub fn get_csv_filename(&self) -> String {
        format!(
            "{}_{}.csv",
            self.dex_type.to_string().to_lowercase(),
            &self.pool_address.to_string()[..8]
        )
    }

    pub fn pool_address(&self) -> &Pubkey {
        &self.pool_address
    }

    pub fn dex_type(&self) -> DexType {
        self.dex_type
    }

    pub fn token_mint(&self) -> &Pubkey {
        &self.token_mint
    }

    pub fn builder() -> PoolConfigBuilder {
        PoolConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct PoolConfigBuilder {
    pool_address: Option<Pubkey>,
    dex_type: Option<DexType>,
    token_mint: Option<Pubkey>,
}

impl PoolConfigBuilder {
    pub fn pool_address(mut self, address: &str) -> Result<Self, AppError> {
        self.pool_address = Some(
            Pubkey::from_str(address)
                .map_err(|e| AppError::ConfigError(format!("Invalid pool address: {}", e)))?,
        );
        Ok(self)
    }

    pub fn pool_address_pubkey(mut self, pubkey: Pubkey) -> Self {
        self.pool_address = Some(pubkey);
        self
    }

    pub fn dex_type(mut self, dex_type: DexType) -> Self {
        self.dex_type = Some(dex_type);
        self
    }

    pub fn dex_type_str(mut self, dex_type: &str) -> Result<Self, AppError> {
        self.dex_type = Some(dex_type.parse()?);
        Ok(self)
    }

    pub fn token_mint(mut self, mint: &str) -> Result<Self, AppError> {
        self.token_mint = Some(
            Pubkey::from_str(mint)
                .map_err(|e| AppError::ConfigError(format!("Invalid token mint: {}", e)))?,
        );
        Ok(self)
    }

    pub fn token_mint_pubkey(mut self, pubkey: Pubkey) -> Self {
        self.token_mint = Some(pubkey);
        self
    }

    pub fn build(self) -> Result<PoolConfig, AppError> {
        let pool_address = self
            .pool_address
            .ok_or_else(|| AppError::ConfigError("Pool address is required".to_string()))?;
        let dex_type = self
            .dex_type
            .ok_or_else(|| AppError::ConfigError("DEX type is required".to_string()))?;
        let token_mint = self
            .token_mint
            .ok_or_else(|| AppError::ConfigError("Token mint is required".to_string()))?;

        PoolConfig::new(pool_address, dex_type, token_mint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_validation() {
        let result = PoolConfig::new(
            Pubkey::default(),
            DexType::Raydium,
            Pubkey::new_unique(),
        );
        assert!(result.is_err());

        let result = PoolConfig::new(
            Pubkey::new_unique(),
            DexType::Raydium,
            Pubkey::default(),
        );
        assert!(result.is_err());

        let result = PoolConfig::new(
            Pubkey::new_unique(),
            DexType::Raydium,
            Pubkey::new_unique(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_pool_config_csv_filename() {
        let pool_addr = Pubkey::new_unique();
        let config = PoolConfig::new(
            pool_addr,
            DexType::PumpFun,
            Pubkey::new_unique(),
        )
        .unwrap();

        let filename = config.get_csv_filename();
        assert!(filename.starts_with("pumpfun_"));
        assert!(filename.ends_with(".csv"));
    }

    #[test]
    fn test_pool_config_builder() {
        let pool_addr = Pubkey::new_unique();
        let token_mint = Pubkey::new_unique();
        
        let config = PoolConfig::builder()
            .pool_address_pubkey(pool_addr)
            .dex_type(DexType::Raydium)
            .token_mint_pubkey(token_mint)
            .build()
            .unwrap();

        assert_eq!(config.pool_address(), &pool_addr);
        assert_eq!(config.dex_type(), DexType::Raydium);
        assert_eq!(config.token_mint(), &token_mint);
    }

    #[test]
    fn test_pool_config_builder_missing_field() {
        let result = PoolConfig::builder()
            .pool_address_pubkey(Pubkey::new_unique())
            .dex_type(DexType::Raydium)
            .build();
        
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_config_builder_string_input() {
        let pool_addr = Pubkey::new_unique();
        let token_mint = Pubkey::new_unique();
        
        let result = PoolConfig::builder()
            .pool_address(&pool_addr.to_string())
            .unwrap()
            .dex_type_str("raydium")
            .unwrap()
            .token_mint(&token_mint.to_string())
            .unwrap()
            .build();

        assert!(result.is_ok());
    }
}
