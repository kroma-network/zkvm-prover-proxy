use anyhow::Result;
use kroma_prover_proxy::types::{ProofResult, WitnessResult};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofFixture {
    pub program_key: String,
    pub public_values: String,
    pub proof: String,
}

impl ProofFixture {
    fn from_proof_result(proof_result: &ProofResult) -> Self {
        Self {
            program_key: proof_result.program_key.clone(),
            public_values: proof_result.public_values.clone(),
            proof: proof_result.proof.clone(),
        }
    }

    pub fn save_proof<T: ToString>(proof_data: T, proof_result: &ProofResult) -> Result<()> {
        let proof_fixture = ProofFixture::from_proof_result(proof_result);

        let proof_json = serde_json::to_string_pretty(&proof_fixture)?;
        let mut file = File::create(proof_data.to_string())?;
        file.write_all(proof_json.as_bytes())?;
        println!("Proof was saved");

        Ok(())
    }
}

pub fn load_witness<T: ToString>(witness_data: T) -> Result<WitnessResult> {
    let file = File::open(witness_data.to_string())?;
    let reader = std::io::BufReader::new(file);
    let witness_result = serde_json::from_reader(reader)?;

    Ok(witness_result)
}
