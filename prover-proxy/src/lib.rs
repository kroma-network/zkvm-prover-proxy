pub mod errors;
pub mod interface;
pub mod proof_db;
pub mod types;
pub mod utils;
pub mod version;

use alloy_primitives::{hex::FromHex, B256};
use once_cell::sync::Lazy;
use sp1_sdk::{HashableKey, ProverClient, SP1VerifyingKey};

pub const FAULT_PROOF_ELF: &[u8] = include_bytes!("../../program/elf/fault-proof-elf");
pub static VERIFYING_KEY: Lazy<SP1VerifyingKey> = Lazy::new(|| {
    let prover = ProverClient::from_env();
    let (_, vkey) = prover.setup(FAULT_PROOF_ELF);
    vkey
});

pub static VERIFICATION_KEY_HASH: Lazy<B256> = Lazy::new(|| {
    let vkey_str = VERIFYING_KEY.bytes32();
    B256::from_hex(&vkey_str).unwrap()
});

// NOTE(Ethan): equals to `DEFAULT_NETWORK_RPC_URL`` in sp1/creates/sdk/src/network/mod.rs
pub const DEFAULT_NETWORK_RPC_URL: &str = "https://rpc.production.succinct.xyz/";
pub const DEFAULT_PROOF_STORE_PATH: &str = "data/proof_store";