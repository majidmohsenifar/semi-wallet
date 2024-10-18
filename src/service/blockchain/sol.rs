use super::{error::BlockchainError, service::BlockchainConfig};

pub struct SolHandler {
    cfg: BlockchainConfig,
}

impl SolHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        SolHandler { cfg }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
