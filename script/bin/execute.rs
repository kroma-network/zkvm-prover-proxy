use clap::Parser;
use script::FAULT_PROOF_ELF;
use std::path::PathBuf;

use kroma_prover_proxy::utils::load_witness;
use sp1_sdk::{utils as sdk_utils, SP1Stdin};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// L2 block number for derivation.
    #[arg(long)]
    witness_path: PathBuf,
}

fn main() {
    let args = Args::parse();
    sdk_utils::setup_logger();

    let wr = load_witness(&args.witness_path.to_str().unwrap().to_string()).unwrap();
    let mut sp1_stdin = SP1Stdin::default();
    sp1_stdin.buffer = wr.get_witness_buf();

    let prover = sp1_sdk::ProverClient::from_env();
    let result = prover.execute(FAULT_PROOF_ELF, &sp1_stdin).run().unwrap();
    println!("Execution report: {:?}", result);
}
