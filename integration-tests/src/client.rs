use alloy_primitives::B256;
use anyhow::Result;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee_core::{client::ClientT, rpc_params};

use kroma_prover_proxy::{
    errors::ProverError,
    types::{ProofResult, RequestResult as ProverRequest, SpecResult as ProverSpec, WitnessResult},
};
use std::time::Duration;

const CLIENT_TIMEOUT_SEC: u64 = 10800;
const DEFAULT_PROVER_RPC_ENDPOINT: &str = "http://0.0.0.0:3031";

pub struct TestClient {
    prover_client: HttpClient,
}

impl TestClient {
    pub fn new(prover_proxy_url: &str) -> Self {
        let prover_client = HttpClientBuilder::default()
            .max_request_body_size(300 * 1024 * 1024)
            .request_timeout(Duration::from_secs(CLIENT_TIMEOUT_SEC))
            .build(prover_proxy_url)
            .unwrap();

        Self { prover_client }
    }
}

impl Default for TestClient {
    fn default() -> Self {
        let prover_client = HttpClientBuilder::default()
            .max_request_body_size(300 * 1024 * 1024)
            .request_timeout(Duration::from_secs(CLIENT_TIMEOUT_SEC))
            .build(DEFAULT_PROVER_RPC_ENDPOINT)
            .unwrap();

        Self { prover_client }
    }
}

impl TestClient {
    pub async fn prover_spec(&self) -> ProverSpec {
        let params = rpc_params![];
        self.prover_client.request("spec", params).await.unwrap()
    }

    pub async fn request_prove(
        &self,
        l2_hash: B256,
        l1_head_hash: B256,
        witness_result: &WitnessResult,
    ) -> Result<ProverRequest, ProverError> {
        let params = rpc_params![l2_hash, l1_head_hash, &witness_result.witness];
        match self.prover_client.request("requestProve", params).await {
            Ok(result) => Ok(result),
            Err(e) if e.to_string().contains("Invalid parameters") => {
                Err(ProverError::invalid_input_hash("Invalid parameters".to_string()))
            }
            Err(e) if e.to_string().contains("SP1 NETWORK ERROR") => {
                // TODO: correct error message for `sp1_network_error`
                Err(ProverError::sp1_network_error(e.to_string()))
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    pub async fn get_proof(&self, l2_hash: B256, l1_head_hash: B256) -> ProofResult {
        let params = rpc_params![l2_hash, l1_head_hash];
        self.prover_client.request("getProof", params).await.unwrap()
    }
}
