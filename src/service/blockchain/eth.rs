use super::{error::BlockchainError, service::BlockchainConfig};

pub struct EthHandler {
    cfg: BlockchainConfig,
}

impl EthHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        EthHandler { cfg }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
