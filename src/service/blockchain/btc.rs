use serde::Deserialize;

use super::{error::BlockchainError, service::BlockchainConfig};

const ADDRESS_URI: &str = "api/v2/address";

pub struct BtcHandler {
    cfg: BlockchainConfig,
    http_client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct GetAddressResponse {
    pub balance: String,
}

impl BtcHandler {
    pub fn new(cfg: BlockchainConfig, http_client: reqwest::Client) -> Self {
        BtcHandler { cfg, http_client }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        let response = self
            .http_client
            .get(format!("{}/{}/{}", self.cfg.url, ADDRESS_URI, addr))
            .send()
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot call the btc node".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        let body = response
            .bytes()
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot read the response of the btc node".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        let res: GetAddressResponse =
            serde_json::from_slice(&body).map_err(|e| BlockchainError::Unexpected {
                message: "cannot parse the response".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        let b: f64 = res
            .balance
            .parse()
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot parse string balance".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        Ok(b / 10u32.pow(self.cfg.decimals as u32) as f64)
    }

    pub async fn get_token_balance(&self, _: &str, _: &str, _: u8) -> Result<f64, BlockchainError> {
        Err(BlockchainError::TokenNotSupported {
            blockchain: "BTC".to_string(),
        })
    }
}
