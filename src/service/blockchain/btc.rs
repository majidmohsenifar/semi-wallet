use super::{error::BlockchainError, service::BlockchainConfig};

pub struct BtcHandler {
    cfg: BlockchainConfig,
}

impl BtcHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        BtcHandler { cfg }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        unimplemented!()
    }
}
