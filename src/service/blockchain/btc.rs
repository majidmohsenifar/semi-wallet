use super::{
    error::BlockchainError,
    service::{BlockchainConfig, BlockchainHandler},
};

pub struct BtcHandler {
    cfg: BlockchainConfig,
}

impl BtcHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        BtcHandler { cfg }
    }
}

impl BlockchainHandler for BtcHandler {
    fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }

    fn get_token_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
