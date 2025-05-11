use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use ripemd::Ripemd160;
use base58::ToBase58;
use secp256k1::{Secp256k1, PublicKey, Message};
use secp256k1::ecdsa::Signature;
use chrono::Utc;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub txid: String,
    pub index: usize,
    pub signature: String,
    pub pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: u64,
    pub address: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl TxInput {
    /// Verifica se a entrada é válida comparando assinatura e endereço
    pub fn is_valid(&self, original_output: &TxOutput) -> bool {
        let secp = Secp256k1::new();

        let pubkey_bytes = match hex::decode(&self.pubkey) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let pubkey = match PublicKey::from_slice(&pubkey_bytes) {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Deriva o endereço da pubkey
        let sha256 = Sha256::digest(&pubkey.serialize());
        let ripemd = Ripemd160::digest(&sha256);
        let mut payload = vec![0x00];
        payload.extend(&ripemd);
        let checksum = &Sha256::digest(&Sha256::digest(&payload))[0..4];
        payload.extend(checksum);
        let derived_address = payload.to_base58();

        if derived_address != original_output.address {
            return false;
        }

        // Recria a mensagem e verifica a assinatura
        let msg_hash = Sha256::digest(format!("{}{}", self.txid, self.index).as_bytes());
        let msg = match Message::from_digest_slice(&msg_hash) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let sig_bytes = match hex::decode(&self.signature) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let signature = match Signature::from_der(&sig_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        secp.verify_ecdsa(&msg, &signature, &pubkey).is_ok()
    }
}

impl Transaction {
    /// Cria uma transação normal com inputs/outputs
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        let mut tx = Transaction {
            id: String::new(),
            inputs,
            outputs,
        };
        tx.id = tx.calculate_hash();
        tx
    }

    /// Cria uma transação coinbase (bloco de mineração)
    pub fn new_coinbase(to_address: &str, reward: u64) -> Self {
        let input = TxInput {
            txid: "0".to_string(),
            index: 0,
            signature: "coinbase".to_string(),
            pubkey: "coinbase".to_string(),
        };
        let output = TxOutput {
            value: reward,
            address: to_address.to_string(),
            timestamp: Utc::now().timestamp(),
            
        };
        Transaction::new(vec![input], vec![output])
    }

    /// Gera o hash da transação
    fn calculate_hash(&self) -> String {
        let raw = format!("{:?}{:?}", self.inputs, self.outputs);
        let hash = Sha256::digest(raw.as_bytes());
        format!("{:x}", hash)
    }
}
