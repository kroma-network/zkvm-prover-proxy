pub mod errors;
pub mod interface;
pub mod proof_db;
pub mod types;
pub mod utils;
pub mod version;

use once_cell::sync::Lazy;
use sp1_sdk::{HashableKey, ProverClient};

pub const FAULT_PROOF_ELF: &[u8] = include_bytes!("../../program/elf/fault-proof-elf");
pub static PROGRAM_KEY: Lazy<String> = Lazy::new(|| {
    let prover = ProverClient::from_env();
    let (_, vkey) = prover.setup(FAULT_PROOF_ELF);
    vkey.bytes32()
});

// NOTE(Ethan): equals to `DEFAULT_NETWORK_RPC_URL`` in sp1/creates/sdk/src/network/mod.rs
pub const DEFAULT_NETWORK_RPC_URL: &str = "https://rpc.production.succinct.xyz/";
pub const DEFAULT_PROOF_STORE_PATH: &str = "data/proof_store";