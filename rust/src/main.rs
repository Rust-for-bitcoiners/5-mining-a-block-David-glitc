use std::fs::File;
use std::fs;
use serde::{Deserialize, Serialize};
use std::io::Write;
use sha2::{Sha256, Digest};



const DIFFICULTY_TARGET: &str = "0000ffff00000000000000000000000000000000000000000000000000000000";

#[derive(Serialize, Deserialize, Debug)]
struct PrevOut {
    scriptpubkey: String,
    scriptpubkey_asm: String,
    scriptpubkey_type: String,
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vin {
    txid: String,
    vout: u64,
    prevout: PrevOut,
    scriptsig: String,
    scriptsig_asm: String,
    witness: Option<Vec<String>>,
    is_coinbase: bool,
    sequence: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vout {
    scriptpubkey: String,
    scriptpubkey_asm: String,
    scriptpubkey_type: String,
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    txid: String,
    version: u64,
    locktime: u64,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
    size: u64,
    weight: u64,
    fee: u64,
    status: Status,
    hex: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    confirmed: bool,
    block_height: u64,
    block_hash: String,
    block_time: u64,
}

fn validate_transaction(tx: &Transaction) -> bool {
    !tx.vin.is_empty() && !tx.vout.is_empty() && tx.status.confirmed
}


// read Transaction Files
fn read_transactions_from_mempool() -> Vec<Transaction> {
    let mempool_dir = "mempool";
    let mut transactions = Vec::new();

    for entry in fs::read_dir(mempool_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension() == Some(std::ffi::OsStr::new("json")) {
            let file_content = fs::read_to_string(path).unwrap();
            let tx: Result<Transaction, _> = serde_json::from_str(&file_content);
            match tx {
                Ok(tx) => {
                    // Process the transaction
                    transactions.push(tx);
                },
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}", e);
                }
            }

        }
    }

    transactions
}


//
fn create_coinbase_transaction() -> Transaction {
    Transaction {
        txid: "coinbase_txid".to_string(),
        version: 1,
        locktime: 0,
        vin: vec![], // Coinbase has no inputs
        vout: vec![Vout {
            scriptpubkey: "5120deef3731bdcccf281c4800106602316ecdfb41e67097243522f910241bffc616".to_string(),
            scriptpubkey_asm: "OP_PUSHNUM_1 OP_PUSHBYTES_32 deef3731bdcccf281c4800106602316ecdfb41e67097243522f910241bffc616".to_string(),
            scriptpubkey_type: "v1_p2tr".to_string(),
            scriptpubkey_address: Some("miner_address".to_string()),
            value: 50_000_000, // Example reward in satoshis
        }],
        size: 0,
        weight: 0,
        fee: 0,
        status: Status {
            confirmed: true,
            block_height: 0,
            block_hash: String::new(),
            block_time: 0,
        },
        hex: String::new(),
    }
}


//
#[derive(Serialize, Deserialize, Debug)]
struct Block {
    header: String,
    transactions: Vec<Transaction>,
}

fn create_block(transactions: Vec<Transaction>) -> Block {
    let coinbase_transaction = create_coinbase_transaction();
    let mut all_transactions = vec![coinbase_transaction];
    all_transactions.extend(transactions);

    Block {
        header: "previous_block_hash".to_string(),
        transactions: all_transactions,
    }
}



fn write_block_to_file(block: &Block) {
    let mut file = File::create("out.txt").unwrap();
    writeln!(file, "{}", block.header).unwrap();

    // Write the serialized coinbase transaction
    writeln!(file, "{:?}", block.transactions[0]).unwrap();

    // Write all transaction IDs (coinbase first)
    for tx in &block.transactions {
        writeln!(file, "{}", tx.txid).unwrap();
    }
}

fn mine_block(header: &str) -> (u64, String) {
    let mut nonce = 0;
    loop {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", header, nonce));
        let result = hasher.finalize();
        let hash_hex = format!("{:x}", result);

        if hash_hex < DIFFICULTY_TARGET.to_string(){
            return (nonce, hash_hex);
        }
        nonce += 1;
    }
}
fn main() {
    // Read transactions from mempool
    let transactions = read_transactions_from_mempool();

    //  Validate transactions
    let valid_transactions = transactions.into_iter().filter(validate_transaction).collect::<Vec<_>>();

    //  Create a block with valid transactions
    let mut block = create_block(valid_transactions);

    //  Mine the block
    let (nonce, block_hash) = mine_block(&block.header);

    block.header = block_hash;

    // Write the block to out.txt
    write_block_to_file(&block);

    println!("Block mined successfully with nonce: {}", nonce);
}