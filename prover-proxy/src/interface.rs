use jsonrpc_core::Result as JsonResult;
use jsonrpc_derive::rpc;
use kroma_zkvm_common::types::preprocessing;
use sp1_sdk::network::NetworkClient;
use std::sync::{Arc, RwLock};

use crate::errors::ProverError;
use crate::proof_db::ProofDB;
use crate::types::{ProofResult, RequestResult, SpecResult};

use crate::{DEFAULT_NETWORK_RPC_URL, DEFAULT_PROOF_STORE_PATH};

#[rpc]
pub trait Rpc {
    #[rpc(name = "spec")]
    fn spec(&self) -> JsonResult<SpecResult>;

    #[rpc(name = "requestProve")]
    fn request_prove(
        &self,
        l2_hash: String,
        l1_head_hash: String,
        witness: String,
    ) -> JsonResult<RequestResult>;

    #[rpc(name = "getProof")]
    fn get_proof(&self, l2_hash: String, l1_head_hash: String) -> JsonResult<ProofResult>;
}

#[derive(Clone)]
pub struct RpcImpl {
    task_lock: Arc<RwLock<()>>,
    proof_db: Arc<ProofDB>,
    pub client: Arc<NetworkClient>,
}

impl RpcImpl {
    pub fn new(store_path: &str, sp1_private_key: &str, network_rpc_url: &str) -> Self {
        RpcImpl {
            task_lock: Arc::new(RwLock::new(())),
            proof_db: Arc::new(ProofDB::new(store_path)),
            client: Arc::new(NetworkClient::new(sp1_private_key, network_rpc_url)),
        }
    }
}

impl Default for RpcImpl {
    fn default() -> Self {
        let sp1_private_key = std::env::var("SP1_PRIVATE_KEY")
            .expect("SP1_PRIVATE_KEY must be set for remote proving");
        Self::new(DEFAULT_PROOF_STORE_PATH, &sp1_private_key, DEFAULT_NETWORK_RPC_URL)
    }
}

impl Rpc for RpcImpl {
    fn spec(&self) -> JsonResult<SpecResult> {
        let spec = SpecResult::default();
        tracing::info!("Received sepc: {:?}", spec);
        Ok(spec)
    }

    fn request_prove(
        &self,
        l2_hash: String,
        l1_head_hash: String,
        witness: String,
    ) -> JsonResult<RequestResult> {
        let (l2_hash, l1_head_hash, user_req_id) =
            preprocessing(&l2_hash, &l1_head_hash).map_err(|e| {
                tracing::error!(
                    "Invalid parameters - \"l2_hash\": {:?}, \"l1_head_hash\": {:?}",
                    l2_hash,
                    l1_head_hash
                );
                ProverError::invalid_input_hash(e.to_string()).to_json_error()
            })?;
        tracing::info!("Received request - \"user_req_id\": {:?}", user_req_id);

        // Check a status of the request.
        let _guard = self.task_lock.write().unwrap();
        let req_status = crate::utils::get_status_by_local_id(
            &self.client,
            &self.proof_db,
            &l2_hash,
            &l1_head_hash,
        );
        
        tracing::info!("Check the status of the request: {:?}, {:?}", user_req_id, req_status);
        // Return the status in case of `Processing` or `Completed`.
        if req_status == RequestResult::Processing || req_status == RequestResult::Completed {
            return Ok(req_status);
        }

        // Send a request to the SP1 Network Prover only if the status is `None` or `Failed`.
        let net_req_id =
            crate::utils::request_prove_to_sp1(&self.client, witness).map_err(|e| {
                tracing::error!("Failed to send request to SP1 network: {:?}", e);
                ProverError::sp1_network_error(e.to_string()).to_json_error()
            })?;
        tracing::info!("Sent request to SP1 network: {:?}, {:?}", user_req_id, net_req_id);

        // Store the `net_req_id` to the database.
        self.proof_db.set_request_id(&l2_hash, &l1_head_hash, &net_req_id).unwrap();
        tracing::info!("Stored \"net_req_id\" to db: {:?}, {:?}", user_req_id, net_req_id);

        Ok(RequestResult::Processing)
    }

    fn get_proof(&self, l2_hash: String, l1_head_hash: String) -> JsonResult<ProofResult> {
        let (l2_hash, l1_head_hash, user_req_id) =
            preprocessing(&l2_hash, &l1_head_hash).map_err(|e| {
                tracing::error!(
                    "Invalid parameters - \"l2_hash\": {:?}, \"l1_head_hash\": {:?}",
                    l2_hash,
                    l1_head_hash
                );
                ProverError::invalid_input_hash(e.to_string()).to_json_error()
            })?;
        tracing::info!("Received get - \"user_req_id\": {:?}", user_req_id);

        // Check if the proof is already stored.
        let guard = self.task_lock.read().unwrap();
        if let Some(proof) =
            crate::utils::get_proof_by_local_id(&self.proof_db, &l2_hash, &l1_head_hash)
        {
            tracing::info!("Proof was found in db: {:?}", user_req_id);
            return Ok(ProofResult::new(
                &self.proof_db.get_request_id(&l2_hash, &l1_head_hash).unwrap(),
                RequestResult::Completed,
                proof,
            ));
        }
        tracing::info!("Proof is not in db: {:?}", user_req_id);
        drop(guard);

        // Check if it has been requested.
        let _guard = self.task_lock.write().unwrap();
        let proof_result = match crate::utils::get_status_by_local_id(
            &self.client,
            &self.proof_db,
            &l2_hash,
            &l1_head_hash,
        ) {
            RequestResult::Completed => {
                let proof = self.proof_db.get_proof(&l2_hash, &l1_head_hash).unwrap();
                ProofResult::new(&user_req_id, RequestResult::Completed, proof)
            }
            RequestResult::Processing => ProofResult::processing(user_req_id),
            RequestResult::None => ProofResult::none(),
            RequestResult::Failed => ProofResult::failed(user_req_id),
        };
        tracing::info!("return the proof result: {:?}", proof_result);

        Ok(proof_result)
    }
}
