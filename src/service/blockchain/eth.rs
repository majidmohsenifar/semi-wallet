use super::{
    error::BlockchainError,
    service::{BlockchainConfig, BlockchainHandler},
};

pub struct EthHandler {
    cfg: BlockchainConfig,
    http_client: reqwest::blocking::Client,
}

impl EthHandler {
    pub fn new(cfg: BlockchainConfig, http_client: reqwest::blocking::Client) -> Self {
        EthHandler { cfg, http_client }
    }
}

impl BlockchainHandler for EthHandler {
    fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }

    fn get_token_balance(&self, contract_addr: &str, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
