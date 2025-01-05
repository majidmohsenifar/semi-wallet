use super::{error::BlockchainError, service::BlockchainConfig};

use alloy::{
    hex::FromHex,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::client::RpcClient,
    sol,
    transports::http::{Client, Http},
};
use std::ops::Div;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Token,
    "erc20.abi.json"
);

pub struct EthHandler {
    cfg: BlockchainConfig,
    provider: RootProvider<Http<Client>>,
}

impl EthHandler {
    pub fn new(cfg: BlockchainConfig) -> Result<Self, BlockchainError> {
        let url = reqwest::Url::parse(&cfg.url).map_err(|e| BlockchainError::Unexpected {
            message: "invalid eth node url".into(),
            source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        })?;
        let client = RpcClient::new_http(url);
        let provider = ProviderBuilder::new().on_client(client);
        //let provider = ProviderBuilder::new().on_http(url);
        Ok(EthHandler { cfg, provider })
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        let addr = Address::from_hex(addr).map_err(|_| BlockchainError::InvalidAddress)?;
        let b = self
            .provider
            .get_balance(addr)
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot call api".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        let base = U256::from(10).pow(U256::from(self.cfg.decimals));
        let div = b.div(base); // We divide by 10^18
        let b = f64::from(div);
        Ok(b)
    }

    pub async fn get_token_balance(
        &self,
        contract_addr: &str,
        addr: &str,
        decimals: u8,
    ) -> Result<f64, BlockchainError> {
        let addr = Address::from_hex(addr).map_err(|_| BlockchainError::InvalidAddress)?;
        let contract_addr =
            Address::from_hex(contract_addr).map_err(|e| BlockchainError::Unexpected {
                message: "invalid contract addr".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        let contract = Token::new(contract_addr, &self.provider);

        let b = contract
            .balanceOf(addr)
            .call()
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot call contract".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        let base = U256::from(10).pow(U256::from(decimals));
        let div = b._0.div(base); // We divide by 10^decimals
        let b = f64::from(div);
        Ok(b)
    }
}
