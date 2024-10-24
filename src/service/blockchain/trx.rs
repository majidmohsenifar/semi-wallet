use super::{
    error::BlockchainError,
    service::{BlockchainConfig, BlockchainHandler},
};

pub struct TrxHandler {
    cfg: BlockchainConfig,
    http_client: reqwest::blocking::Client,
}

impl TrxHandler {
    pub fn new(cfg: BlockchainConfig, http_client: reqwest::blocking::Client) -> Self {
        TrxHandler { cfg, http_client }
    }
}

impl BlockchainHandler for TrxHandler {
    fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }

    fn get_token_balance(&self, contract_addr: &str, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
