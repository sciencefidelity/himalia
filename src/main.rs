#![allow(clippy::unwrap_used)]
use std::error::Error;

use data_encoding::HEXLOWER;
use log::LevelFilter;
use structopt::StructOpt;

use himalia::server::{send_tx, Server, CENTRAL_NODE};
use himalia::wallet::{self, validate_address, ADDRESS_CHECK_SUM_LEN};
use himalia::{blockchain::Blockchain, config::GLOBAL_CONFIG};
use himalia::{transactions::Transaction, utxo_set::UTXOSet, wallets::Wallets};

const MINE_TRUE: usize = 1;

#[derive(Debug, StructOpt)]
#[structopt(name = "himalia")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "createblockchain", about = "Create a new blockchain")]
    CreateBlockchain {
        #[structopt(
            name = "address",
            help = "The address to send the genesis block reward to"
        )]
        address: String,
    },
    #[structopt(name = "createwallet", help = "Create a new wallet")]
    CreateWallet,
    #[structopt(
        name = "getbalance",
        about = "Get the wallet balance of the target address"
    )]
    GetBalance {
        #[structopt(name = "address", help = "The wallet address")]
        address: String,
    },
    #[structopt(name = "listaddresses", about = "Pring local wallet address")]
    ListAddresses,
    #[structopt(name = "send", about = "Add new block to chain")]
    Send {
        #[structopt(name = "from", help = "Source wallet address")]
        from: String,
        #[structopt(name = "to", help = "Destination wallet address")]
        to: String,
        #[structopt(name = "amount", help = "Amount to send")]
        amount: i32,
        #[structopt(name = "mine", help = "Mine immediately on the same node")]
        mine: usize,
    },
    #[structopt(name = "printchain", about = "Print blockchain all blocks")]
    PrintChain,
    #[structopt(name = "reindexutxo", about = "Rebuild UTXO index set")]
    ReindexUtxo,
    #[structopt(name = "startnode", about = "Start a node")]
    StartNode {
        #[structopt(name = "miner", help = "Enable mining mode and send rewerd to ADDRESS")]
        miner: Option<String>,
    },
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let opt = Opt::from_args();
    match opt.command {
        Command::CreateBlockchain { address } => {
            let blockchain = Blockchain::create(address.as_str());
            let utxo_set = UTXOSet::new(blockchain);
            utxo_set.reindex();
            println!("Done!");
        }
        Command::CreateWallet => {
            let mut wallet = Wallets::new();
            let address = wallet.create_wallet();
            println!("Your new address: {address}");
        }
        Command::GetBalance { address } => {
            let address_valid = validate_address(address.as_str());
            assert!(address_valid, "Error: Address in not valid");
            let payload = himalia::base58_decode(address.as_str());
            let pub_key_hash = &payload[1..payload.len() - ADDRESS_CHECK_SUM_LEN];

            let blockchain = Blockchain::new();
            let utxo_set = UTXOSet::new(blockchain);
            let utxos = utxo_set.find_utxo(pub_key_hash);
            let mut balance = 0;
            for utxo in utxos {
                balance += utxo.get_value();
            }
            println!("Balance of {address}, {balance}");
        }
        Command::ListAddresses => {
            let wallets = Wallets::new();
            for address in wallets.get_addresses() {
                println!("{address}");
            }
        }
        Command::Send {
            from,
            to,
            amount,
            mine,
        } => {
            assert!(
                validate_address(from.as_str()),
                "Error: Sender address is not valid"
            );
            assert!(
                validate_address(to.as_str()),
                "Error: Recipient address is not valid"
            );
            let blockchain = Blockchain::new();
            let utxo_set = UTXOSet::new(blockchain.clone());

            let transaction =
                Transaction::new_utxo_transaction(from.as_str(), to.as_str(), amount, &utxo_set);

            if mine == MINE_TRUE {
                let coinbase_tx = Transaction::new_coinbase_tx(from.as_str());
                let block = blockchain.mine_block(&[transaction, coinbase_tx]);
                utxo_set.update(&block);
            } else {
                send_tx(CENTRAL_NODE, &transaction)?;
            }
            println!("Success!");
        }
        Command::PrintChain => {
            let mut block_iterator = Blockchain::new().iterator();
            loop {
                let option = block_iterator.next();
                if option.is_none() {
                    break;
                }
                let block = option.unwrap();
                println!("Pre block hash: {}", block.get_pre_block_hash());
                println!("Cur block hash: {}", block.get_hash());
                println!("Pre block timestamp: {}", block.get_timestamp());
                for tx in block.get_transactions() {
                    let cur_txid_hex = HEXLOWER.encode(tx.get_id());
                    println!("– Transaction txid_hex: {cur_txid_hex}");
                    if !tx.is_coinbase() {
                        for input in tx.get_vin() {
                            let txid_hex = HEXLOWER.encode(input.get_txid());
                            let pub_key_hash = wallet::hash_pub_key(input.get_pub_key());
                            let address = wallet::convert_address(pub_key_hash.as_slice());
                            println!(
                                "-- Input txid = {txid_hex}, vout = {}, from = {address}",
                                input.get_vout()
                            );
                        }
                    }
                    for output in tx.get_vout() {
                        let pub_key_hash = output.get_pub_key_hash();
                        let address = wallet::convert_address(pub_key_hash);
                        println!("-- Output value = {}, to = {address}", output.get_value());
                    }
                }
                println!();
            }
        }
        Command::ReindexUtxo => {
            let blockchain = Blockchain::new();
            let utxo_set = UTXOSet::new(blockchain);
            utxo_set.reindex();
            let count = utxo_set.count_transactions();
            println!("Done! There are {count} transactions in the UTXO set.");
        }
        Command::StartNode { miner } => {
            if let Some(addr) = miner {
                assert!(validate_address(addr.as_str()), "Wrong miner address");
                println!("Mining is on. Address to receive rewards: {addr}");
                GLOBAL_CONFIG.set_mining_addr(addr);
            }
            let blockchain = Blockchain::new();
            let socket_addr = GLOBAL_CONFIG.get_node_addr();
            Server::new(blockchain).run(socket_addr.as_str())?;
        }
    }
    Ok(())
}
