use ethers::prelude::*;
use ethers::types::Address;
use prettytable::{Cell, Row, Table};
use reqwest;
use rusqlite::{params, Connection, Result};
use std::convert::TryFrom;
use telegram_bot::*;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     dotenv::dotenv().ok();
//
//     // Connect to the SQLite database (it will be created if it doesn't exist)
//     let conn = Connection::open("erc20_contracts.db")?;
//
//     // Create the table if it doesn't exist
//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS contracts (
//                   id INTEGER PRIMARY KEY,
//                   contract_address TEXT NOT NULL,
//                   owner_address TEXT NOT NULL,
//                   owner_name TEXT,
//                   token_name TEXT,
//                   decimals INTEGER,
//                   total_supply TEXT,
//                   status TEXT NOT NULL
//                   )",
//         params![],
//     )?;
//
//     let rpcProviderAPI = std::env::var("RPC_PROVIDER_API").expect("Missing RPC_PROVIDER_API env var");
//     let telegramAPIKey = std::env::var("TELEGRAM_API_KEY").expect("Missing TELEGRAM_API_KEY env var");
//     let etherscanAPIKey = std::env::var("ETHERSCAN_API_KEY").expect("Missing ETHERSCAN_API_KEY env var");
//
//     // Create a new http provider
//     let provider = Provider::<Http>::try_from(rpcProviderAPI)?; // replace with your Ethereum node URL
//     let provider = provider.interval(Duration::from_secs(6u64));
//
//     // Define the ERC20 Transfer event
//     #[derive(Debug, Clone, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
//     struct Transfer {
//         from: Address,
//         to: Address,
//         value: U256,
//     }
//     impl Event for Transfer {
//         const EVENT_SIGNATURE: &'static str =
//             "Transfer(address indexed from,address indexed to,uint256 value)";
//         const TOPICS: &'static [&'static str] = &["from", "to"];
//     }
//
//     // Create a filter for this event with 0x0 as the `from` address
//     let filter = Transfer::filter()
//         .from(Some(vec![Address::zero()]))
//         .to(None)
//         .value(None);
//
//     // Create a new Telegram bot
//     let api = Api::new(telegramAPIKey); // replace with your Telegram bot token
//
//     // Create a new chat ID (replace with your Telegram chat ID)
//     let chat = ChatId::new(1635189176);
//
//     // Subscribe to the event
//     let mut stream = provider.subscribe_logs(filter).await?;
//
//     // Create an ABI instance to interact with the contract
//     let abi = Abi::from_str(erc20_abi::ERC20_ABI)?; // replace with the correct ABI
//
//     // For each new event
//     while let Some(log) = stream.next().await {
//         // Use a match statement to handle errors gracefully
//         match log {
//             Ok(log) => {
//                 if let Ok(event) = Transfer::parse_log(log.clone()) {
//                     println!("New ERC20 contract: {}", log.address);
//                     println!("Owner address: {}", event.to);
//
//                     // Connect to the contract
//                     let contract = Contract::new(log.address, abi.clone(), provider.clone());
//
//                     // Call the contract functions and handle possible errors
//                     let result = (|| async {
//                         let name: String = contract.method("name", ())?.call().await?;
//                         let decimals: U8 = contract.method("decimals", ())?.call().await?;
//                         let total_supply: U256 = contract.method("totalSupply", ())?.call().await?;
//
//                         // Get the owner name from Etherscan
//                         let client = reqwest::Client::new();
//                         let etherscan_api_key = etherscanAPIKey;  // Replace with your Etherscan API key
//                         let url = format!("https://api.etherscan.io/api?module=account&action=getAddressInfo&address={}&tag=latest&apikey={}", event.to, etherscan_api_key);
//                         let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
//                         let owner_name = resp["result"]["tag"].as_str().unwrap_or("Unknown");
//
//                         // Save the data to the SQLite database
//                         conn.execute(
//                             "INSERT INTO contracts (contract_address, owner_address, owner_name, token_name, decimals, total_supply, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
//                             params![log.address.to_string(), event.to.to_string(), owner_name, name, decimals, total_supply.to_string(), "Created"],
//                         )?;
//
//                         // Query the database and print all contracts in a pretty table
//                         let mut stmt = conn.prepare("SELECT * FROM contracts")?;
//                         let contracts_iter = stmt.query_map(params![], |row| {
//                             Ok((
//                                 row.get(0)?,
//                                 row.get(1)?,
//                                 row.get(2)?,
//                                 row.get(3)?,
//                                 row.get(4)?,
//                                 row.get(5)?,
//                                 row.get(6)?,
//                                 row.get(7)?,
//                             ))
//                         })?;
//
//                         let mut table = Table::new();
//                         table.add_row(row!["ID", "Contract Address", "Owner Address", "Owner Name", "Token Name", "Decimals", "Total Supply", "Status"]);
//                         for contract in contracts_iter {
//                             let (id, contract_address, owner_address, owner_name, token_name, decimals, total_supply, status): (i32, String, String, String, String, i32, String, String) = contract?;
//                             table.add_row(Row::new(vec![
//                                 Cell::new(&id.to_string()),
//                                 Cell::new(&contract_address),
//                                 Cell::new(&owner_address),
//                                 Cell::new(&owner_name),
//                                 Cell::new(&token_name),
//                                 Cell::new(&decimals.to_string()),
//                                 Cell::new(&total_supply),
//                                 Cell::new(&status),
//                             ]));
//                         }
//                         table.printstd();
//
//                         // Send a message to the Telegram chat
//                         let text = format!(
//                             "New ERC20 contract: {}\n\
//                             Owner address: {} ({})\n\
//                             Name: {}\n\
//                             Decimals: {}\n\
//                             Total Supply: {}",
//                             log.address, event.to, owner_name, name, decimals, total_supply
//                         );
//                         api.send(chat.text(text)).await?;
//                         Ok::<(), Box<dyn std::error::Error>>(())
//                     })().await;
//
//                     // If there was an error, print it and continue
//                     if let Err(e) = result {
//                         println!("Error processing transaction: {:?}", e);
//                     }
//                 }
//             },
//             Err(e) => println!("Error getting log: {:?}", e),
//         }
//     }
//
//     Ok(())
// }

fn main() {
    println!("Hello, world!");
}
