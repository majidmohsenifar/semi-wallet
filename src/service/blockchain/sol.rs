use std::str::FromStr;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;

use super::{error::BlockchainError, service::BlockchainConfig};

pub struct SolHandler {
    cfg: BlockchainConfig,
    client: RpcClient,
}

impl SolHandler {
    pub fn new(cfg: BlockchainConfig) -> Self {
        let client = RpcClient::new_with_commitment(cfg.url.clone(), CommitmentConfig::confirmed());
        SolHandler { cfg, client }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        let pub_key = Pubkey::from_str(addr).map_err(|_e| BlockchainError::InvalidAddress)?;
        let b = self.client.get_balance(&pub_key).await.map_err(|e| {
            println!("error: {:?}", e);
            BlockchainError::Unexpected {
                message: "cannot get balance".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            }
        })?;
        Ok(b as f64 / self.cfg.decimals as f64)
    }

    pub async fn get_token_balance(
        &self,
        contract_addr: &str,
        addr: &str,
        _: u8,
    ) -> Result<f64, BlockchainError> {
        let mint_key =
            Pubkey::from_str(contract_addr).map_err(|_e| BlockchainError::InvalidAddress)?;
        let pub_key = Pubkey::from_str(addr).map_err(|_e| BlockchainError::InvalidAddress)?;
        let token_address = get_associated_token_address(&pub_key, &mint_key);
        let b = self
            .client
            .get_token_account_balance(&token_address)
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot get token balance".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        let b = b.ui_amount.ok_or(BlockchainError::InvalidAddress)?;
        Ok(b)
    }
}
