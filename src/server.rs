use std::{
    error::Error,
    io::{BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use crate::{
    block::Block,
    blockchain::Blockchain,
    config::GLOBAL_CONFIG,
    memory_pool::{BlockInTransit, MemoryPool},
    node::Nodes,
    transaction::Transaction,
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

const NODE_VERSION: usize = 1;
pub const CENTRAL_NODE: &str = "127.0.0.1:2001";
pub const TRANSACTION_THRESHOLD: usize = 2;
static GLOBAL_NODES: Lazy<Nodes> = Lazy::new(|| {
    let nodes = Nodes::new();
    nodes.add_node(String::from(CENTRAL_NODE));
    nodes
});
static GLOBAL_MEMORY_POOL: Lazy<MemoryPool> = Lazy::new(|| MemoryPool::new());
static GLOBAL_BLOCKS_IN_TRANSIT: Lazy<BlockInTransit> = Lazy::new(|| BlockInTransit::new());
const TCP_WRITE_TIMEOUT: u64 = 1000;

/// Defines essential functionalities to handle incoming client connections,
/// communicate with a central node, and concurrently manage requests from
/// multiple clients through separate threads.
pub struct Server {
    blockchain: Blockchain,
}

impl Server {
    /// Initializes a new `Server` with the provided blockchain.
    pub fn new(blockchain: Blockchain) -> Self {
        Self { blockchain }
    }

    pub fn run(&self, addr: &str) {
        let listener = TcpListener::bind(addr).unwrap();
        if addr.eq(CENTRAL_NODE) == false {
            let best_height = self.blockchain.get_best_height();
            send_version(CENTRAL_NODE, best_height);
        }
        for stream in listener.incoming() {
            let blockchain = self.blockchain.clone();
            thread::spawn(|| match stream {
                Ok(stream) => {
                    //
                }
                Err(e) => {
                    //
                }
            });
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OpType {
    /// Operations related to transactions.
    Tx,
    /// Activities linked to blocks in the blockchain.
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
/// request to the specified address in the blockchain network.
fn send_get_data(addr: &str, op_type: OpType, id: &[u8]) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::GetData {
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
/// using a standardized package format, which in this case represents blocks.
/// Will help broadcast inventory notifications for specific data items to the
/// indicated network address.
fn send_inv(addr: &str, op_type: OpType, blocks: &[Vec<u8>]) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Inv {
            addr_from: node_addr,
            op_type,
            items: blocks.to_vec(),
        },
    )?;
    Ok(())
}

/// Transmits a block to a specified network address.
///
/// Abstracts the process of sending a block to a specified address using
/// a standardized package format. The block is serialized before sending, likely
/// to transmit it efficiently in byte form over the network.
fn send_block(addr: &str, block: &Block) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Block {
            addr_from: node_addr,
            block: block.serialize(),
        },
    )?;
    Ok(())
}

/// Dispatches a transaction to a specified network address.
///
/// Abstracts the process of sending a transaction to a specified address using
/// a standardized package format. The transaction is serialized before sending
/// for efficient transmission over the network.
fn send_tx(addr: &str, tx: &Transaction) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Tx {
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
/// the node's version and the best-known height.
fn send_version(addr: &str, height: usize) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Version {
            addr_from: node_addr,
            version: NODE_VERSION,
            best_height: height,
        },
    )?;
    Ok(())
}

/// Transmits a request for block data to a specified network address.
///
/// Abstracts the process of sending a request for blocks to a specified address
/// using a standardized package format. The request does not include any specific
/// block IDs or other parameters, it simply requests blocks from the receiving node.
fn send_get_blocks(addr: &str) -> Result<(), Box<dyn Error>> {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::GetBlocks {
            addr_from: node_addr,
        },
    )?;
    Ok(())
}

fn serve(blockchain: Blockchain, stream: TcpStream) -> Result<(), Box<dyn Error>> {
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
                if GLOBAL_BLOCKS_IN_TRANSIT.len() > 0 {
                    let block_hash = GLOBAL_BLOCKS_IN_TRANSIT.first().unwrap();
                    send_get_data(addr_from.as_str(), OpType::Block, &block_hash);
                    GLOBAL_BLOCKS_IN_TRANSIT.remove(block_hash.as_slice());
                }
            }
        }
    }
}

/// Sends data packages to a specified socket address.
fn send_data(addr: SocketAddr, pkg: Package) -> Result<(), Box<dyn Error>> {
    info!("send package: {:?}", &pkg);
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
