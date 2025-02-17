use anyhow::Result;
use clap::Parser;
use jsonrpc_http_server::ServerBuilder;
use kroma_prover_proxy::{
    interface::{Rpc, RpcImpl}, utils::block_on, DEFAULT_NETWORK_RPC_URL, DEFAULT_PROOF_STORE_PATH, FAULT_PROOF_ELF, VERIFICATION_KEY_HASH, VERIFYING_KEY
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long = "endpoint", default_value = "0.0.0.0:3031")]
    endpoint: String,

    #[clap(short, long = "data", default_value = DEFAULT_PROOF_STORE_PATH)]
    data_path: String,
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::Subscriber::builder().init();

    let args = Args::parse();

    let sp1_private_key =
        std::env::var("SP1_PRIVATE_KEY").expect("SP1_PRIVATE_KEY must be set for remote proving");
    let rpc_impl = RpcImpl::new(&args.data_path, &sp1_private_key, DEFAULT_NETWORK_RPC_URL);
    
    block_on(async {
        let vk_hash = rpc_impl.client.register_program(&VERIFYING_KEY, FAULT_PROOF_ELF).await.unwrap();
        tracing::info!("The program’s key was retrieved from the network: {:?}", vk_hash);
    });
    
    let mut io = jsonrpc_core::IoHandler::new();
    io.extend_with(rpc_impl.to_delegate());
    
    tracing::info!("Starting Prover at {}", args.endpoint);
    tracing::info!("Program Key: {:#?}", VERIFICATION_KEY_HASH.to_string());
    let server = ServerBuilder::new(io)
        .threads(3)
        .max_request_body_size(200 * 1024 * 1024)
        .start_http(&args.endpoint.parse().unwrap())
        .unwrap();

    server.wait();

    Ok(())
}
