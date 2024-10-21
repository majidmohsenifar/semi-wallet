use super::{
    error::BlockchainError,
    service::{BlockchainConfig, BlockchainHandler},
};

pub struct SolHandler {
    cfg: BlockchainConfig,
}

impl SolHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        SolHandler { cfg }
    }
}

impl BlockchainHandler for SolHandler {
    fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }

    fn get_token_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
