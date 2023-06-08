use std::error::Error;
use std::sync::Arc;
use ethers::abi::{AbiDecode, Address};
use ethers::prelude::*;
// use ethers::providers::{Http, Middleware, Provider, StreamExt, Ws};
use eyre::Result;

#[derive(Debug, Clone, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct ERC20Token {
    name: String,
    symbol: String,
    decimals: u8,
    contract_address: Address,
    owner_address: Address,
    status: String
}

impl ERC20Token {
    fn new(name: String, symbol: String, decimals: u8, contract_address: Address, owner_address: Address, status: String) -> Self {
        Self {
            name,
            symbol,
            decimals,
            contract_address,
            owner_address,
            status,
        }
    }
}

abigen!(
        ERC20Contract,
        r#"[
            function balanceOf(address account) external view returns (uint256)
            function decimals() external view returns (uint8)
            function symbol() external view returns (string memory)
            function name() external view returns (string memory)
            function transfer(address to, uint256 amount) external returns (bool)
            event Transfer(address indexed from, address indexed to, uint256 value)
            event OwnershipTransferred(address,address)
        ]"#,
);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let rpc_provider_api = std::env::var("RPC_PROVIDER_API").expect("Missing RPC_PROVIDER_API env var");
    let wss_provider_api = std::env::var("WSS_PROVIDER_API").expect("Missing WSS_PROVIDER_API env var");
    let telegram_apikey = std::env::var("TELEGRAM_API_KEY").expect("Missing TELEGRAM_API_KEY env var");
    let etherscan_apikey = std::env::var("ETHERSCAN_API_KEY").expect("Missing ETHERSCAN_API_KEY env var");

    println!("rpc_provider_api: {}, wss_provideR_api: {}", rpc_provider_api, wss_provider_api);
    let provider = Provider::<Ws>::connect(wss_provider_api).await?;
    let rpcProvider = Provider::<Http>::try_from(rpc_provider_api)?;

    let last_block = provider.get_block(BlockNumber::Latest).await?.unwrap().number.unwrap();
    println!("last_block: {last_block}");

    let ownership_transfer_filter = Filter::new()
        .topic1(Address::zero())
        .event("OwnershipTransferred(address,address)");

    let mut stream = provider.subscribe_logs(&ownership_transfer_filter).await?;

    while let Some(log) = stream.next().await {
        let contract_address = log.address;
        match get_token_info(&rpcProvider, log, contract_address).await {
            Ok(token) => {
                println!("get token: {:?}", token);
            },
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn get_token_info(provider: &Provider<Http>, log: Log, contract_address: Address) -> Result<ERC20Token, Box<dyn Error>> {

    let token_contract = ERC20Contract::new(contract_address, Arc::new(provider.clone()));
    let name = token_contract.name().call().await?;
    let symbol = token_contract.symbol().call().await?;
    let decimals = token_contract.decimals().call().await?;

    let token = ERC20Token::new(name, symbol, decimals, contract_address, Address::zero(), "new".to_string());
    println!(
        "block: {:?}, tx: {:?}, token: {:?}",
        log.block_number,
        log.transaction_hash,
        token
    );
    Ok(token)
}