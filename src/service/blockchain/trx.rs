use base58::FromBase58;
use serde::{Deserialize, Serialize};

use super::{error::BlockchainError, service::BlockchainConfig};

pub struct TrxHandler {
    cfg: BlockchainConfig,
    http_client: reqwest::Client,
}

#[derive(Serialize)]
pub struct GetAccountRequestBody<'a> {
    address: &'a str,
    visible: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct GetAccountResponseBody {
    address: String,
    balance: u64,
    create_time: u64,
    latest_opration_time: u64,
    latest_consume_free_time: u64,
}

#[derive(Serialize)]
struct TriggerConstantContractRequestBody<'a> {
    owner_address: &'a str,
    contract_address: &'a str,
    function_selector: &'a str,
    parameter: String,
    visible: bool,
}

#[derive(Deserialize)]
struct TriggerConstantContractResultResponse {
    result: bool,
}

#[derive(Deserialize)]
struct TriggerConstantContractResponseBody {
    result: TriggerConstantContractResultResponse,
    constant_result: Vec<String>,
}

impl TrxHandler {
    pub fn new(cfg: BlockchainConfig, http_client: reqwest::Client) -> Self {
        TrxHandler { cfg, http_client }
    }

    pub async fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        //GetAccountURI                = "/wallet/getaccount"
        let body = GetAccountRequestBody {
            address: addr,
            visible: true,
        };
        let response = self
            .http_client
            .post(format!("{}/wallet/getaccount", self.cfg.url))
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
        Ok(res.balance as f64 / self.cfg.decimals as f64)
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
            owner_address: addr,
            contract_address: contract_addr,
            function_selector: "balanceOf(address)",
            parameter,
            visible: true,
        };
        let response = self
            .http_client
            .post(format!("{}/wallet/triggerconstantcontract", self.cfg.url))
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
        Ok(balance as f64 / decimals as f64)
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
