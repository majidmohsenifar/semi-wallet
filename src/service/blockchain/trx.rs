use super::{
    error::BlockchainError,
    service::{BlockchainConfig, BlockchainHandler},
};

pub struct TrxHandler {
    cfg: BlockchainConfig,
}

impl TrxHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        TrxHandler { cfg }
    }
}

impl BlockchainHandler for TrxHandler {
    fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }

    fn get_token_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
