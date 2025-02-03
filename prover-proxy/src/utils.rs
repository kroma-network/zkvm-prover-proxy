use alloy_primitives::B256;
use anyhow::Result;
use sp1_sdk::{
    network::{
        NetworkClient,
        proto::network::{ProofMode, FulfillmentStatus},
    },
    SP1_CIRCUIT_VERSION as SP1_SDK_VERSION, {SP1ProofWithPublicValues, SP1Stdin},
};
use std::sync::Arc;

use crate::{proof_db::ProofDB, types::{RequestResult, WitnessResult}, MAX_CYCLES, VERIFICATION_KEY_HASH};

pub fn block_on<T>(fut: impl std::future::Future<Output = T>) -> T {
    use tokio::task::block_in_place;

    // Handle case if we're already in an tokio runtime.
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        block_in_place(|| handle.block_on(fut))
    } else {
        // Otherwise create a new runtime.
        let rt = tokio::runtime::Runtime::new().expect("Failed to create a new runtime");
        rt.block_on(fut)
    }
}

pub fn request_prove_to_sp1(client: &Arc<NetworkClient>, witness: String) -> Result<B256> {
    // Recover a SP1Stdin from the witness string.
    let mut sp1_stdin = SP1Stdin::new();
    sp1_stdin.buffer = WitnessResult::string_to_witness_buf(&witness);

    // Send a request to generate a proof to the sp1 network.
    tracing::debug!("ready to send request to SP1 network prover");
    
    let response = block_on(async move {
        client
            .request_proof(
                *VERIFICATION_KEY_HASH,
                &sp1_stdin,
                ProofMode::Plonk,
                SP1_SDK_VERSION,
                sp1_sdk::network::FulfillmentStrategy::Hosted,
                10,
                MAX_CYCLES,
            )
            .await
    })?;

    let request_id = B256::from_slice(&response.body.unwrap().request_id);
    tracing::debug!("Sent the request to SP1 network prover: {:?}", request_id);

    Ok(request_id)
}

pub fn get_status_by_local_id(
    client: &Arc<NetworkClient>,
    proof_db: &Arc<ProofDB>,
    l2_hash: &B256,
    l1_head_hash: &B256,
) -> RequestResult {
    let request_id = proof_db.get_request_id(l2_hash, l1_head_hash);
    match request_id {
        Some(id) => get_status_by_remote_id(client, proof_db, id),
        None => RequestResult::None,
    }
}

pub fn status_from_i32(value: i32) -> Option<FulfillmentStatus> {
    match value {
        0 => Some(FulfillmentStatus::UnspecifiedFulfillmentStatus),
        1 => Some(FulfillmentStatus::Requested),
        2 => Some(FulfillmentStatus::Assigned),
        3 => Some(FulfillmentStatus::Fulfilled),
        4 => Some(FulfillmentStatus::Unfulfillable),
        _ => None,
    }
}

pub fn get_status_by_remote_id(
    client: &Arc<NetworkClient>,
    proof_db: &Arc<ProofDB>,
    request_id: B256,
) -> RequestResult {
    let (status, maybe_proof) =
        match block_on(async { client.get_proof_request_status(request_id, None).await }) {
            Ok(res) => res,
            Err(_) => return RequestResult::None,
        };

    match status_from_i32(status.fulfillment_status).unwrap() {
        FulfillmentStatus::Fulfilled => {
            proof_db.set_proof(&request_id, &maybe_proof.unwrap()).unwrap();
            RequestResult::Completed
        }
        FulfillmentStatus::Requested | FulfillmentStatus::Assigned => RequestResult::Processing,
        FulfillmentStatus::Unfulfillable => RequestResult::Failed,
        FulfillmentStatus::UnspecifiedFulfillmentStatus => {
            tracing::error!("The proof status is unspecified: {:?}", request_id);
            RequestResult::None
        }
    }
}

pub fn get_proof_by_local_id(
    proof_db: &Arc<ProofDB>,
    l2_hash: &B256,
    l1_head_hash: &B256,
) -> Option<SP1ProofWithPublicValues> {
    let request_id = proof_db.get_request_id(l2_hash, l1_head_hash);
    match request_id {
        Some(id) => proof_db.get_proof_by_id(&id),
        None => None,
    }
}
