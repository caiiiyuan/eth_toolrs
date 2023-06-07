use ethers::abi::{AbiDecode, Address};
use ethers::prelude::*;
use ethers::providers::{Http, Middleware, Provider, StreamExt, Ws};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let rpc_provider_api = std::env::var("RPC_PROVIDER_API").expect("Missing RPC_PROVIDER_API env var");
    let wss_provider_api = std::env::var("WSS_PROVIDER_API").expect("Missing WSS_PROVIDER_API env var");
    let telegram_apikey = std::env::var("TELEGRAM_API_KEY").expect("Missing TELEGRAM_API_KEY env var");
    let etherscan_apikey = std::env::var("ETHERSCAN_API_KEY").expect("Missing ETHERSCAN_API_KEY env var");

    println!("rpc_provider_api: {}, wss_provideR_api: {}", rpc_provider_api, wss_provider_api);
    let provider = Provider::<Ws>::connect(wss_provider_api).await?;

    let last_block = provider.get_block(BlockNumber::Latest).await?.unwrap().number.unwrap();
    println!("last_block: {last_block}");

    let erc20_transfer_filter = Filter::new()
        // .from_block(last_block - 25)
        .topic1(Address::zero())
        .event("Transfer(address,address,uint256)");

    let mut stream = provider.subscribe_logs(&erc20_transfer_filter).await?;

    while let Some(log) = stream.next().await {
        println!(
            "block: {:?}, tx: {:?}, token: {:?}, from: {:?}, to: {:?}, amount: {:?}",
            log.block_number,
            log.transaction_hash,
            log.address,
            Address::from(log.topics[1]),
            Address::from(log.topics[2]),
            U256::decode(log.data)
        );
    }

    Ok(())
}