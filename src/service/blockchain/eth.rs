use super::{error::BlockchainError, service::BlockchainConfig};

pub struct EthHandler {
    cfg: BlockchainConfig,
    http_client: reqwest::Client,
}

impl EthHandler {
    pub fn new(cfg: BlockchainConfig, http_client: reqwest::Client) -> Self {
        EthHandler { cfg, http_client }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }

    pub async fn get_token_balance(
        &self,
        contract_addr: &str,
        addr: &str,
        decimals: u8,
    ) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
