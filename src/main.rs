use structopt::StructOpt;

const MINE_TRUE: usize = 1;

#[derive(Debug, StructOpt)]
#[structopt(name = "himalia")]
struct Opt {
    #[structopt(subcommand)]
    commands: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "createblockchain", about = "Create a new blockchain")]
    CreateBlockChain {
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
    Printchain,
    #[structopt(name = "reindexutxo", about = "Rebuild UTXO index set")]
    Reindexutxo,
    #[structopt(name = "startnode", about = "Start a node")]
    StartNode {
        #[structopt(name = "miner", help = "Enable mining mode and send rewerd to ADDRESS")]
        miner: Option<String>,
    },
}

fn main() {}

