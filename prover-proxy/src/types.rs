use serde::{Deserialize, Serialize};
use sp1_sdk::{SP1ProofWithPublicValues, SP1_CIRCUIT_VERSION as SP1_SDK_VERSION};

use crate::{version::PROVER_PROXY_VERSION, PROGRAM_KEY};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecResult {
    pub version: String,
    pub sp1_version: String,
    pub program_key: String,
}

impl SpecResult {
    pub fn new(version: String, sp1_version: String, program_key: String) -> Self {
        Self {
            version,
            sp1_version,
            program_key,
        }
    }
}

impl Default for SpecResult {
    fn default() -> Self {
        SpecResult::new(
            PROVER_PROXY_VERSION.to_string(),
            SP1_SDK_VERSION.to_string(),
            PROGRAM_KEY.to_string(),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RequestResult {
    None,
    Processing,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofResult {
    pub request_id: String,
    pub request_status: RequestResult,
    pub program_key: String,
    pub public_values: String,
    pub proof: String,
}

impl ProofResult {
    pub fn new<T: ToString>(
        request_id: &T,
        request_status: RequestResult,
        proof: SP1ProofWithPublicValues,
    ) -> Self {
        Self {
            request_id: request_id.to_string(),
            request_status,
            program_key: PROGRAM_KEY.to_string(),
            public_values: hex::encode(&proof.public_values),
            proof: hex::encode(proof.bytes()),
        }
    }

    pub fn none() -> Self {
        Self {
            request_id: "".to_string(),
            request_status: RequestResult::None,
            program_key: PROGRAM_KEY.to_string(),
            public_values: "".to_string(),
            proof: "".to_string(),
        }
    }

    pub fn processing(request_id: String) -> Self {
        Self {
            request_id,
            request_status: RequestResult::Processing,
            program_key: PROGRAM_KEY.to_string(),
            public_values: "".to_string(),
            proof: "".to_string(),
        }
    }

    pub fn failed(request_id: String) -> Self {
        Self {
            request_id,
            request_status: RequestResult::Failed,
            program_key: PROGRAM_KEY.to_string(),
            public_values: "".to_string(),
            proof: "".to_string(),
        }
    }
}

/// The result of a witness method.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WitnessResult {
    pub status: RequestResult,
    pub program_key: String,
    pub witness: String,
}

impl Default for WitnessResult {
    fn default() -> Self {
        Self::new(RequestResult::None, "".to_string())
    }
}

impl WitnessResult {
    pub const EMPTY_WITNESS: Vec<Vec<u8>> = Vec::new();

    pub fn new<T: ToString>(status: RequestResult, witness: T) -> Self {
        Self {
            status,
            program_key: PROGRAM_KEY.to_string(),
            witness: witness.to_string(),
        }
    }

    pub fn new_with_status(status: RequestResult) -> Self {
        Self::new(status, "".to_string())
    }

    // Note(Ethan): `sp1-core-machine::SP1Stdin` has witness as `Vec<Vec<u8>>`.
    pub fn new_from_witness_buf(status: RequestResult, buf: Vec<Vec<u8>>) -> Self {
        let serialized_witness = bincode::serialize(&buf).unwrap();
        let hex_encoded_with_prefix = "0x".to_string() + hex::encode(&serialized_witness).as_ref();
        Self::new(status, hex_encoded_with_prefix)
    }

    pub fn string_to_witness_buf(witness: &str) -> Vec<Vec<u8>> {
        let witness = hex::decode(witness.strip_prefix("0x").unwrap()).unwrap();
        bincode::deserialize(&witness).unwrap()
    }

    pub fn get_witness_buf(&self) -> Vec<Vec<u8>> {
        Self::string_to_witness_buf(&self.witness)
    }
}
