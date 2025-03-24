mod client;
mod utils;

use alloy_primitives::{b256, B256};
use anyhow::Result;
use client::TestClient;
use kroma_prover_proxy::types::RequestResult as ProverRequest;
use std::{thread::sleep, time::Duration};
use utils::{load_witness, ProofFixture};

struct TestCtx {
    l2_hash: B256,
    l1_head_hash: B256,
}

impl TestCtx {
    fn new(l2_hash: B256, l1_head_hash: B256) -> Self {
        Self { l2_hash, l1_head_hash }
    }
}

async fn scenario(
    client: &TestClient,
    l2_hash: B256,
    l1_head_hash: B256,
    witness_data: &str,
    proof_data: &str,
) {
    client.prover_spec().await.unwrap();

    let witness_result = load_witness(witness_data).expect("failed to load witness");

    // The response should be `Processing`.
    let request_result =
        client.request_prove(l2_hash, l1_head_hash, &witness_result).await.unwrap();
    assert_eq!(
        request_result,
        ProverRequest::Processing,
        "Consider removing the witness data and trying again!"
    );

    // The same response is returned for the same request.
    let request_result =
        client.request_prove(l2_hash, l1_head_hash, &witness_result).await.unwrap();
    assert_eq!(request_result, ProverRequest::Processing);

    let proof_result = loop {
        let proof_result = client.get_proof(l2_hash, l1_head_hash).await;
        if proof_result.request_status == ProverRequest::Completed {
            break proof_result;
        }
        if let ProverRequest::Failed = proof_result.request_status {
            panic!("Failed to get proof");
        }
        sleep(Duration::from_secs(20));
    };

    ProofFixture::save_proof(proof_data, &proof_result).unwrap();
}

#[tokio::test]
async fn test_online_scenario() -> Result<()> {
    let metadata =
        cargo_metadata::MetadataCommand::new().exec().expect("Failed to get cargo metadata");
    let proxy_bin_path = metadata.target_directory.join("release/prover-proxy");
    let mut child = tokio::process::Command::new(proxy_bin_path)
        .args(vec!["--data", "data/proof_store"])
        .spawn()?;

    let client = TestClient::default();
    while client.prover_spec().await.is_err() {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    println!("Prover proxy is ready.");

    let ctx = TestCtx::new(
        b256!("c620c1601621527b982fd8a9b781629edad908d7917c043e243f2277a48f561b"),
        b256!("b00118b43ea791285813f88bf1774508b6c495de9ec17f3f58cc810248d15d5d"),
    );
    let result = std::env::current_dir()?;
    println!("Current directory: {:?}", result);
    scenario(
        &client,
        ctx.l2_hash,
        ctx.l1_head_hash,
        "./tests/data/witness.json",
        "./tests/data/proof.json",
    )
    .await;

    let mut sys = sysinfo::System::new();
    sys.refresh_all();
    if let Some(pid) = child.id() {
        for process in sys.processes().values() {
            if let Some(parent_pid) = process.parent() {
                if parent_pid.as_u32() == pid {
                    process.kill();
                }
            }
        }
        child.kill().await?
    }

    std::fs::remove_dir_all("data")?;

    Ok(())
}
