use std::collections::HashMap;

use crate::config;
use crate::repository::models::Coin;
use crate::service::coin::price_storage::PriceData;
use crate::service::coin::price_storage::PriceStorage;

use super::binance_price_provider::BinancePriceProvider;
use super::error::CoinError;

pub const PRICE_PROVIDER_BINANCE: &str = "BINANCE";

enum Provider {
    Binance,
}

impl Provider {
    pub fn from(value: &str) -> Result<Self, CoinError> {
        match value.to_uppercase().as_str() {
            PRICE_PROVIDER_BINANCE => Ok(Self::Binance),
            _ => Err(CoinError::InvalidPriceProvider),
        }
    }
}

enum PriceProvider {
    Binance(BinancePriceProvider),
}

pub struct PriceManager {
    price_storage: PriceStorage,
}

impl PriceManager {
    pub fn new(price_storage: PriceStorage) -> Self {
        Self { price_storage }
    }

    pub async fn run_update_prices(
        &self,
        price_provider_name: &str,
        coins: Vec<Coin>,
        binance_cfg: config::BinanceConfig,
    ) {
        let provider = Provider::from(price_provider_name).unwrap();
        let price_provider = match provider {
            Provider::Binance => PriceProvider::Binance(BinancePriceProvider::new(
                self.price_storage.clone(),
                coins,
                binance_cfg.ws_url,
            )),
        };
        match &price_provider {
            PriceProvider::Binance(provider) => provider.run_update_prices().await,
        }
    }

    pub async fn get_prices_for_coins<'a>(
        &self,
        symbols: Vec<&'a str>,
    ) -> Result<HashMap<&'a str, PriceData>, CoinError> {
        let result = self
            .price_storage
            .get_prices_for_symbols(symbols)
            .await
            .map_err(|e| {
                tracing::error!(" cannot get_prices_for_symbols due to err: {}", e);
                CoinError::Unexpected {
                    message: "cannot get prices for symbols".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                }
            })?;
        Ok(result)
    }
}
