use super::{error::BlockchainError, service::BlockchainConfig};

pub struct TrxHandler {
    cfg: BlockchainConfig,
}

impl TrxHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        TrxHandler { cfg }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
