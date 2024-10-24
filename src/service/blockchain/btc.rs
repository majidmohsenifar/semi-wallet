use serde::Deserialize;

use super::{
    error::BlockchainError,
    service::{BlockchainConfig, BlockchainHandler},
};

pub struct BtcHandler {
    cfg: BlockchainConfig,
    http_client: reqwest::blocking::Client,
}

impl BtcHandler {
    pub fn new(cfg: BlockchainConfig, http_client: reqwest::blocking::Client) -> Self {
        BtcHandler { cfg, http_client }
    }
}

#[derive(Debug, Deserialize)]
struct BtcNodeResponse {
    result: f64,
    error: Option<String>,
    id: String,
}

#[derive(Debug, Deserialize)]
struct BtcNodeRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: Vec<String>,
}

impl BlockchainHandler for BtcHandler {
    fn get_balance(&self, addr: &str) -> Result<f64, BlockchainError> {
        //TODO: handle this later
        unimplemented!()
        //TODO: handle the req body
        //let req_body = "palang";
        //let response = self
        //.http_client
        //////TODO: what to update here for uri
        //.post(format!("{}/", self.cfg.url))
        //.json(req_body)
        //.send()
        //.map_err(|e| BlockchainError::Unexpected {
        //message: "cannot call the btc node".to_string(),
        //source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        //})?;

        //let body = response.bytes().map_err(|e| BlockchainError::Unexpected {
        //message: "cannot read the response of the btc node".to_string(),
        //source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        //})?;
        //let res: BtcNodeResponse =
        //serde_json::from_slice(&body).map_err(|e| BlockchainError::Unexpected {
        //message: "cannot parse the response".to_string(),
        //source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        //})?;

        //Ok(res.result / 10u32.pow(self.cfg.decimals as u32) as f64)

        ////$ curl -u "$bitcoinauth" -d '{"jsonrpc": "1.0", "id": "curltest", "method": "getbalance", "params": ["*", 6]}' -H 'content-type: text/plain;' http://127.0.0.1:18332/wallet/test-wallet
        ////{"result":0.00000000,"error":null,"id":"curltest"}
        //Ok(2.0)
    }

    fn get_token_balance(&self, _: &str, _: &str) -> Result<f64, BlockchainError> {
        Err(BlockchainError::TokenNotSupported {
            blockchain: "BTC".to_string(),
        })
    }
}
