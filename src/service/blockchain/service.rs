use std::collections::HashMap;

use crate::{config::Settings, repository::models::Coin};

use super::{
    btc::BtcHandler, error::BlockchainError, eth::EthHandler, sol::SolHandler, trx::TrxHandler,
};

pub const BLOCKCHAIN_BTC: &str = "BTC";
pub const BLOCKCHAIN_ETH: &str = "ETH";
pub const BLOCKCHAIN_SOL: &str = "SOL";
pub const BLOCKCHAIN_TRX: &str = "TRX";

pub struct Service {
    handlers: HashMap<Blockchain, Box<dyn BlockchainHandler>>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Blockchain {
    BTC,
    ETH,
    SOL,
    TRX,
}

#[derive(Debug)]
pub struct BlockchainConfig {
    pub url: String,
    pub decimals: u8,
    pub blockbook_support: bool,
}

impl Blockchain {
    pub fn from(value: &str) -> Result<Self, BlockchainError> {
        match value.to_uppercase().as_str() {
            BLOCKCHAIN_BTC => Ok(Self::BTC),
            BLOCKCHAIN_ETH => Ok(Self::ETH),
            BLOCKCHAIN_SOL => Ok(Self::SOL),
            BLOCKCHAIN_TRX => Ok(Self::TRX),
            _ => Err(BlockchainError::InvalidBlockchain),
        }
    }
}

pub trait BlockchainHandler {
    fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError>;
    fn get_token_balance(&self, contract_addr: &str, addr: &str) -> Result<f64, BlockchainError>;
}

impl Service {
    pub fn new(settings: Settings) -> Self {
        let btc_handler = BtcHandler::new(BlockchainConfig {
            url: settings.btc.url,
            decimals: settings.btc.decimals,
            blockbook_support: settings.btc.blockbook_support,
        });
        let eth_handler = EthHandler::new(BlockchainConfig {
            url: settings.eth.url,
            decimals: settings.eth.decimals,
            blockbook_support: settings.eth.blockbook_support,
        });
        let sol_handler = SolHandler::new(BlockchainConfig {
            url: settings.sol.url,
            decimals: settings.sol.decimals,
            blockbook_support: settings.sol.blockbook_support,
        });
        let trx_handler = TrxHandler::new(BlockchainConfig {
            url: settings.trx.url,
            decimals: settings.trx.decimals,
            blockbook_support: settings.trx.blockbook_support,
        });
        let handlers = HashMap::from([
            (
                Blockchain::BTC,
                Box::new(btc_handler) as Box<dyn BlockchainHandler>,
            ),
            (
                Blockchain::ETH,
                Box::new(eth_handler) as Box<dyn BlockchainHandler>,
            ),
            (
                Blockchain::ETH,
                Box::new(sol_handler) as Box<dyn BlockchainHandler>,
            ),
            (Blockchain::ETH, Box::new(trx_handler)),
        ]);
        Service { handlers }
    }

    pub async fn get_balance(&self, coin: &Coin, addr: &str) -> Result<f64, BlockchainError> {
        let blockchain = Blockchain::from(&coin.network)?;
        let blockchain_handler = self
            .handlers
            .get(&blockchain)
            .ok_or(BlockchainError::InvalidBlockchain)?;

        let balance = if coin.contract_address.is_some() {
            let contract_address = coin.contract_address.as_ref();
            blockchain_handler.get_token_balance(contract_address.unwrap(), addr)?
        } else {
            blockchain_handler.get_balance(addr)?
        };
        Ok(balance)
    }
}
