// O que ele já faz: Estrutura completa de um bloco (Block)
//  Prova de trabalho simples (PoW) com SHA-256 Minera bloco até encontrar um hash com prefixo "0000" 
// Garante timestamp real e cálculo de hash único

// vitabit/src/block.rs

// vitabit/src/block.rs

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Representa um bloco individual da blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
    pub data: String,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, data: String) -> Self {
        let timestamp = now();
        let mut nonce = 0;
        let mut hash;

        loop {
            hash = calculate_hash(index, timestamp, &previous_hash, nonce, &data);
            if hash.starts_with("0000") {
                break;
            }
            // 👇 Adicione esta linha para ver o progresso a cada 100 mil tentativas
            if nonce % 100_000 == 0 {
                println!("Tentando nonce {} → hash: {}", nonce, hash);
            }
            nonce += 1;
        }

        Block {
            index,
            timestamp,
            previous_hash,
            nonce,
            hash,
            data,
        }
    }

    pub fn new_difficulty(index: u64, previous_hash: String, data: String, difficulty: usize) -> Self {
        let timestamp = now();
        let mut nonce = 0;
        let mut hash;

        loop {
            hash = calculate_hash(index, timestamp, &previous_hash, nonce, &data);
            if hash.starts_with(&"0".repeat(difficulty)) {
                break;
            }
            nonce += 1;
        }

        Block {
            index,
            timestamp,
            previous_hash,
            nonce,
            hash,
            data,
        }
    }
}

/// Calcula o hash de um bloco com SHA-256
pub fn calculate_hash(index: u64, timestamp: u128, previous_hash: &str, nonce: u64, data: &str) -> String {
    let input = format!("{}{}{}{}{}", index, timestamp, previous_hash, nonce, data);
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}

/// Retorna o timestamp atual em milissegundos
fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = Block::new(0, String::from("0"), String::from("Genesis Block"));
        assert!(block.hash.starts_with("0000"));
        assert_eq!(block.index, 0);
        assert_eq!(block.previous_hash, "0");
        assert_eq!(block.data, "Genesis Block");
    }
}
