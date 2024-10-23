use super::{
    error::BlockchainError,
    service::{BlockchainConfig, BlockchainHandler},
};

pub struct EthHandler {
    cfg: BlockchainConfig,
}

impl EthHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        EthHandler { cfg }
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
