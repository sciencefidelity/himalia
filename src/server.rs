use std::io::{BufReader, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::{error::Error, thread, time::Duration};

use data_encoding::HEXLOWER;
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::memory_pool::{BlockInTransit, MemoryPool};
use crate::transactions::Transaction;
use crate::utxo_set::UTXOSet;
use crate::{block::Block, blockchain::Blockchain, config::GLOBAL_CONFIG, node::Nodes};

const NODE_VERSION: usize = 1;
pub const CENTRAL_NODE: &str = "127.0.0.1:2001";
pub const TRANSACTION_THRESHOLD: usize = 2;
static GLOBAL_NODES: Lazy<Nodes> = Lazy::new(|| {
    let nodes = Nodes::new();
    nodes.add_node(String::from(CENTRAL_NODE));
    nodes
});
static GLOBAL_MEMORY_POOL: Lazy<MemoryPool> = Lazy::new(MemoryPool::new);
static GLOBAL_BLOCKS_IN_TRANSIT: Lazy<BlockInTransit> = Lazy::new(BlockInTransit::new);
const TCP_WRITE_TIMEOUT: u64 = 1000;

/// Defines essential functionalities to handle incoming client connections,
/// communicate with a central [Node], and concurrently manage requests from
/// multiple clients through separate threads.
pub struct Server {
    blockchain: Blockchain,
}

impl Server {
    /// Initializes a new [Server] with the provided [Blockchain].
    pub const fn new(blockchain: Blockchain) -> Self {
        Self { blockchain }
    }

    pub fn run(&self, addr: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(addr).unwrap();
        if !addr.eq(CENTRAL_NODE) {
            let best_height = self.blockchain.get_best_height();
            send_version(CENTRAL_NODE, best_height)?;
        }
        for stream in listener.incoming() {
            let _blockchain = self.blockchain.clone();
            thread::spawn(|| match stream {
                Ok(_stream) => {
                    //
                }
                Err(_e) => {
                    //
                }
            });
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OpType {
    /// Operations related to [Transaction]s.
    Tx,
    /// Activities linked to [Blocks] in the [Blockchain].
    Block,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Package {
    Block {
        addr_from: String,
        block: Vec<u8>,
    },
    GetBlocks {
        addr_from: String,
    },
    GetData {
        addr_from: String,
        op_type: OpType,
        id: Vec<u8>,
    },
    Inv {
        addr_from: String,
        op_type: OpType,
        items: Vec<Vec<u8>>,
    },
    Tx {
        addr_from: String,
        transaction: Vec<u8>,
    },
    Version {
        addr_from: String,
        version: usize,
        best_height: usize,
    },
}

/// Transmits a request for specific data to a designated network address.
///
/// Abstracts the process of sending a specific type of data to a specified
/// address using a standardized package format. Will initiate a data retrieval
/// request to the specified address in the [Blockchain] network.
fn send_get_data(addr: &str, op_type: OpType, id: &[u8]) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        &Package::GetData {
            addr_from: node_addr,
            op_type,
            id: id.to_vec(),
        },
    )?;
    Ok(())
}

/// Notifies about specific data items to a provided network address.
///
/// Abstracts the process of sending inventory information to a specified address
/// using a standardized package format, which in this case represents [Block]s.
/// Will help broadcast inventory notifications for specific data items to the
/// indicated network address.
fn send_inv(addr: &str, op_type: OpType, blocks: &[Vec<u8>]) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        &Package::Inv {
            addr_from: node_addr,
            op_type,
            items: blocks.to_vec(),
        },
    )?;
    Ok(())
}

/// Transmits a [Block] to a specified network address.
///
/// Abstracts the process of sending a block to a specified address using
/// a standardized package format. The block is serialized before sending, likely
/// to transmit it efficiently in byte form over the network.
fn send_block(addr: &str, block: &Block) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        &Package::Block {
            addr_from: node_addr,
            block: block.serialize(),
        },
    )?;
    Ok(())
}

/// Dispatches a [Transaction] to a specified network address.
///
/// Abstracts the process of sending a [Transaction] to a specified address using
/// a standardized package format. The [Transaction] is serialized before sending
/// for efficient transmission over the network.
pub fn send_tx(addr: &str, tx: &Transaction) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        &Package::Tx {
            addr_from: node_addr,
            transaction: tx.serialize(),
        },
    )?;
    Ok(())
}

/// Broadcasts version information to a specified network address.
///
/// Abstracts the process of sending a version message to a specified address using
/// a standardized package format. The version message includes information about
/// the [Node]'s version and the best-known height.
fn send_version(addr: &str, height: usize) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        &Package::Version {
            addr_from: node_addr,
            version: NODE_VERSION,
            best_height: height,
        },
    )?;
    Ok(())
}

/// Transmits a request for [Block] data to a specified network address.
///
/// Abstracts the process of sending a request for blocks to a specified address
/// using a standardized package format. The request does not include any specific
/// block IDs or other parameters, it simply requests blocks from the receiving node.
fn send_get_blocks(addr: &str) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        &Package::GetBlocks {
            addr_from: node_addr,
        },
    )?;
    Ok(())
}

/// Receives a TCP connection and a [Blockchain] instance. Deserializes incoming packages
/// from the stream and processes them based on their type.
// TODO: Split this up!
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
pub fn serve(blockchain: &Blockchain, stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let peer_addr = stream.peer_addr()?;
    let reader = BufReader::new(&stream);
    let pkg_reader = Deserializer::from_reader(reader).into_iter::<Package>();
    for pkg in pkg_reader {
        let pkg = pkg?;
        info!("Receive request from {peer_addr}: {pkg:?}");
        match pkg {
            Package::Block { addr_from, block } => {
                let block = Block::deserialize(block.as_slice());
                blockchain.add_block(&block);
                info!("Added block {}", block.get_hash());
                if !GLOBAL_BLOCKS_IN_TRANSIT.is_empty() {
                    let block_hash = GLOBAL_BLOCKS_IN_TRANSIT.first().unwrap();
                    send_get_data(addr_from.as_str(), OpType::Block, &block_hash)?;
                    GLOBAL_BLOCKS_IN_TRANSIT.remove(block_hash.as_slice());
                }
            }
            Package::GetBlocks { addr_from } => {
                let blocks = blockchain.get_block_hashes();
                send_inv(addr_from.as_str(), OpType::Block, &blocks)?;
            }
            Package::GetData {
                addr_from,
                op_type,
                id,
            } => match op_type {
                OpType::Block => {
                    if let Some(block) = blockchain.get_block(id.as_slice()) {
                        send_block(addr_from.as_str(), &block)?;
                    }
                }
                OpType::Tx => {
                    let txid_hex = HEXLOWER.encode(id.as_slice());
                    if let Some(tx) = GLOBAL_MEMORY_POOL.get(txid_hex.as_str()) {
                        send_tx(addr_from.as_str(), &tx)?;
                    }
                }
            },
            Package::Inv {
                addr_from,
                op_type,
                items,
            } => match op_type {
                OpType::Block => {
                    GLOBAL_BLOCKS_IN_TRANSIT.add_blocks(items.as_slice());
                    let block_hash = items.first().unwrap();
                    send_get_data(addr_from.as_str(), OpType::Block, block_hash)?;
                    GLOBAL_BLOCKS_IN_TRANSIT.remove(block_hash);
                }
                OpType::Tx => {
                    let txid = items.first().unwrap();
                    let txid_hex = HEXLOWER.encode(txid);
                    if !GLOBAL_MEMORY_POOL.contains(txid_hex.as_str()) {
                        send_get_data(addr_from.as_str(), OpType::Tx, txid)?;
                    }
                }
            },
            Package::Tx {
                addr_from,
                transaction,
            } => {
                let tx = Transaction::deserialize(transaction.as_slice());
                let txid = tx.get_id_bytes();
                GLOBAL_MEMORY_POOL.add(tx);
                let node_addr = GLOBAL_CONFIG.get_node_addr();
                if node_addr.eq(CENTRAL_NODE) {
                    let nodes = GLOBAL_NODES.get_nodes();
                    for node in &nodes {
                        if node_addr.eq(node.get_addr().as_str()) {
                            continue;
                        }
                        if addr_from.eq(node.get_addr().as_str()) {
                            continue;
                        }
                        send_inv(node.get_addr().as_str(), OpType::Tx, &[txid.clone()])?;
                    }
                }
                if GLOBAL_MEMORY_POOL.len() >= TRANSACTION_THRESHOLD && GLOBAL_CONFIG.is_miner() {
                    let mining_address = GLOBAL_CONFIG.get_mining_addr().unwrap();
                    let coinbase_tx = Transaction::new_coinbase_tx(mining_address.as_str());
                    let mut txs = GLOBAL_MEMORY_POOL.get_all();
                    txs.push(coinbase_tx);
                    let new_block = blockchain.mine_block(&txs);
                    let utxo_set = UTXOSet::new(blockchain.clone());
                    utxo_set.reindex();
                    info!("New block {} is mined!", new_block.get_hash());
                    for tx in &txs {
                        let txid_hex = HEXLOWER.encode(tx.get_id());
                        GLOBAL_MEMORY_POOL.remove(txid_hex.as_str());
                    }
                    let nodes = GLOBAL_NODES.get_nodes();
                    for node in &nodes {
                        if node_addr.eq(node.get_addr().as_str()) {
                            continue;
                        }
                        send_inv(
                            node.get_addr().as_str(),
                            OpType::Block,
                            &[new_block.get_hash_bytes()],
                        )?;
                    }
                }
            }
            Package::Version {
                addr_from,
                version,
                best_height,
            } => {
                info!("version = {version}, best_height = {best_height}");
                let local_best_height = blockchain.get_best_height();
                if local_best_height < best_height {
                    send_get_blocks(addr_from.as_str())?;
                }
                if local_best_height > best_height {
                    send_version(addr_from.as_str(), blockchain.get_best_height())?;
                }
                if !GLOBAL_NODES.node_is_known(peer_addr.to_string().as_str()) {
                    GLOBAL_NODES.add_node(addr_from);
                }
            }
        }
    }
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

/// Sends data packages to a specified socket address.
fn send_data(addr: SocketAddr, pkg: &Package) -> Result<(), Box<dyn Error>> {
    info!("send package: {:?}", pkg);
    let stream = TcpStream::connect(addr);
    if stream.is_err() {
        error!("The {addr} is not valid");
        GLOBAL_NODES.evict_node(addr.to_string().as_str());
        return Ok(());
    }
    let mut stream = stream.unwrap();
    stream.set_write_timeout(Option::from(Duration::from_millis(TCP_WRITE_TIMEOUT)))?;
    serde_json::to_writer(&stream, &pkg)?;
    stream.flush()?;
    Ok(())
}
