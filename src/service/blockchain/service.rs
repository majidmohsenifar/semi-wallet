use std::collections::HashMap;

use crate::config::Settings;

use super::{
    btc::BtcHandler, error::BlockchainError, eth::EthHandler, sol::SolHandler, trx::TrxHandler,
};

pub const BLOCKCHAIN_BTC: &str = "BTC";
pub const BLOCKCHAIN_ETH: &str = "ETH";
pub const BLOCKCHAIN_SOL: &str = "SOL";
pub const BLOCKCHAIN_TRX: &str = "TRX";

pub struct Service {
    handlers: HashMap<Blockchain, BlockchainHandler>,
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

enum BlockchainHandler {
    Btc(BtcHandler),
    Eth(EthHandler),
    Sol(SolHandler),
    Trx(TrxHandler),
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
            (Blockchain::BTC, BlockchainHandler::Btc(btc_handler)),
            (Blockchain::ETH, BlockchainHandler::Eth(eth_handler)),
            (Blockchain::SOL, BlockchainHandler::Sol(sol_handler)),
            (Blockchain::TRX, BlockchainHandler::Trx(trx_handler)),
        ]);
        Service { handlers }
    }

    pub async fn get_balance(
        &self,
        blockchain: Blockchain,
        addr: &str,
    ) -> Result<f64, BlockchainError> {
        let blockchain_handler = self
            .handlers
            .get(&blockchain)
            .ok_or(BlockchainError::InvalidBlockchain)?;

        let balance = match blockchain_handler {
            BlockchainHandler::Btc(handler) => handler.get_balance(addr).await,
            BlockchainHandler::Eth(handler) => handler.get_balance(addr).await,
            BlockchainHandler::Sol(handler) => handler.get_balance(addr).await,
            BlockchainHandler::Trx(handler) => handler.get_balance(addr).await,
        }?;
        Ok(balance)
    }
}
