use base58::FromBase58;
use serde::{Deserialize, Serialize};

use super::{error::BlockchainError, service::BlockchainConfig};

pub const GET_ACCOUNT_URI: &str = "/wallet/getaccount";
pub const TRIGGER_SMART_CONTRACT_URI: &str = "/wallet/triggerconstantcontract";

pub struct TrxHandler {
    cfg: BlockchainConfig,
    http_client: reqwest::Client,
}

#[derive(Serialize, Deserialize)]
pub struct GetAccountRequestBody {
    pub address: String,
    pub visible: bool,
}

#[derive(Deserialize)]
pub struct GetAccountResponseBody {
    pub address: String,
    pub balance: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TriggerConstantContractRequestBody {
    pub owner_address: String,
    pub contract_address: String,
    pub function_selector: String,
    pub parameter: String,
    pub visible: bool,
}

#[derive(Deserialize, Serialize)]
pub struct TriggerConstantContractResultResponse {
    pub result: bool,
}

#[derive(Deserialize, Serialize)]
pub struct TriggerConstantContractResponseBody {
    pub result: TriggerConstantContractResultResponse,
    pub constant_result: Vec<String>,
}

impl TrxHandler {
    pub fn new(cfg: BlockchainConfig, http_client: reqwest::Client) -> Self {
        TrxHandler { cfg, http_client }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        let body = GetAccountRequestBody {
            address: addr.to_string(),
            visible: true,
        };
        let response = self
            .http_client
            .post(format!("{}{}", self.cfg.url, GET_ACCOUNT_URI))
            .json(&body)
            .send()
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot call api".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        if !response.status().is_success() {
            return Err(BlockchainError::UnsuccessfulStatusCode {
                code: response.status().as_u16(),
            });
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot read body bytes".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        let res: GetAccountResponseBody =
            serde_json::from_slice(&bytes).map_err(|e| BlockchainError::Unexpected {
                message: "cannot deserialize the response".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        Ok(res.balance as f64 / 10_i32.pow(self.cfg.decimals as u32) as f64)
    }

    pub async fn get_token_balance(
        &self,
        contract_addr: &str,
        addr: &str,
        decimals: u8,
    ) -> Result<f64, BlockchainError> {
        let parameter = get_hex_address(addr).map_err(|e| BlockchainError::Unexpected {
            message: "cannot get hex of address".to_string(),
            source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        })?;
        let body = TriggerConstantContractRequestBody {
            owner_address: addr.to_string(),
            contract_address: contract_addr.to_string(),
            function_selector: "balanceOf(address)".to_string(),
            parameter,
            visible: true,
        };
        let response = self
            .http_client
            .post(format!("{}{}", self.cfg.url, TRIGGER_SMART_CONTRACT_URI))
            .json(&body)
            .send()
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot call api".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        if !response.status().is_success() {
            return Err(BlockchainError::UnsuccessfulStatusCode {
                code: response.status().as_u16(),
            });
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot read body bytes".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        let res: TriggerConstantContractResponseBody =
            serde_json::from_slice(&bytes).map_err(|e| BlockchainError::Unexpected {
                message: "cannot deserialize the response".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        if !res.result.result {
            return Err(BlockchainError::UnsuccessfulTronContractCall);
        }

        let balance_hex = res
            .constant_result
            .first()
            .ok_or(BlockchainError::UnsuccessfulTronContractCall)?;
        let balance = balance_hex
            .parse::<f64>()
            .map_err(|e| BlockchainError::Unexpected {
                message: "cannot convert balance hex to f64".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        Ok(balance as f64 / 10_i32.pow(decimals as u32) as f64)
    }
}

pub fn get_hex_address(addr: &str) -> Result<String, BlockchainError> {
    let hex_addr = addr
        .from_base58()
        .map_err(|_| BlockchainError::InvalidAddress)?;
    //getting from 1 to remove the checksum
    let hex_string = hex::encode(&hex_addr[1..]);
    Ok(format!("{:0>64}", hex_string))
}
