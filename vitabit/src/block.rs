

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::prelude::*;
use crate::transaction::Transaction;


/// Estrutura de um bloco da blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Block {
         pub transactions: Vec<Transaction>,
    pub index: u64,             // Posição do bloco na cadeia
    pub timestamp: i64,         // Timestamp Unix
    pub previous_hash: String,  // Hash do bloco anterior
    pub hash: String,           // Hash atual do bloco (calculado com PoW)
    pub nonce: u64,             // Nonce usado na mineração (PoW)
    pub data: String,           // Conteúdo do bloco (transações em JSON)
    pub extra_reward: u64,      // Recompensa adicional (ex: regra dos 100 anos)
}

impl Block {
    /// Cria novo bloco e o minera com dificuldade
    pub fn new(index: u64, previous_hash: String, data: String, extra_reward: u64) -> Self {
        let timestamp = Utc::now().timestamp();
        let transactions = vec![];
        let mut block = Block {
            index,
            timestamp,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            data,
            extra_reward,
            transactions,
        };
        block.mine(4); // 💡 dificuldade fixa 4 (0000...)
        block
    }

    /// Calcula o hash SHA256 do bloco (baseado nos dados + nonce)
    pub fn calculate_hash(&self) -> String {
        let input = format!(
            "{}{}{}{}{}{}",
            self.index,
            self.timestamp,
            self.previous_hash,
            self.nonce,
            self.data,
            self.extra_reward
        );
        let hash = Sha256::digest(input.as_bytes());
        format!("{:x}", hash)
    }

    /// Minera o bloco até encontrar hash com prefixo de zeros conforme dificuldade
    pub fn mine(&mut self, difficulty: usize) {
        let prefix = "0".repeat(difficulty);
        loop {
            self.hash = self.calculate_hash();
            if &self.hash[..difficulty] == prefix {
                break;
            }
            self.nonce += 1;
        }
    }

    /// Cria o bloco gênesis com mensagem imutável
    pub fn genesis() -> Self {
        let message = "No princípio era o Verbo, imutável como VitaBit. alea jacta est";
        Self::new(0, "0".to_string(), message.to_string(), 0)
    }
}
