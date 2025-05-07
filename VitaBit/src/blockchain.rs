// vitabit/src/blockchain.rs

use crate::block::{Block, calculate_hash};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write, Read};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub block_reward: u64,
}

const BLOCKS_PER_ADJUSTMENT: usize = 2016;
const TARGET_BLOCK_TIME: u128 = 600_000;
const TWO_WEEKS_MS: u128 = 1_209_600_000;
const HALVING_INTERVAL: u64 = 4; // mudar aki posr pronto 210_000
const INITIAL_REWARD: u64 = 50;

impl Blockchain {
    pub fn new() -> Self {
        let genesis_message = "No princípio era o Verbo, imutável como SHA-256. Se teus fundos dormirem por cem anos, à rede retornarão — para renascer. Pois como a Palavra não passa, esta moeda viverá. Nada se perderá. | In the beginning was the Word, immutable as SHA-256. Should thy funds sleep for a hundred years, they shall return to the network — to be reborn. For as the Word endures, so shall this coin live. Nothing shall be lost.";
        let genesis_block = Block::new(0, String::from("0"), genesis_message.to_string());

        Blockchain {
            chain: vec![genesis_block],
            difficulty: 4,
            block_reward: INITIAL_REWARD,
        }
    }

    pub fn add_block(&mut self, data: String) {
        let index = self.chain.len() as u64;
        self.update_reward(index);

        let difficulty = self.current_difficulty();
        let last_hash = self.chain.last().unwrap().hash.clone();

        let new_block = Block::new_difficulty(
            index,
            last_hash,
            format!("{} | reward: {} VBIT", data, self.block_reward),
            difficulty,
        );

        self.chain.push(new_block);
    }

    fn update_reward(&mut self, height: u64) {
        let halvings = height / HALVING_INTERVAL;
        self.block_reward = INITIAL_REWARD >> halvings.min(64);
    }

    pub fn current_difficulty(&self) -> usize {
        let len = self.chain.len();
        if len < BLOCKS_PER_ADJUSTMENT || len % BLOCKS_PER_ADJUSTMENT != 0 {
            return self.difficulty;
        }

        let start = &self.chain[len - BLOCKS_PER_ADJUSTMENT];
        let end = &self.chain[len - 1];
        let actual_time = end.timestamp - start.timestamp;

        if actual_time < TWO_WEEKS_MS / 2 {
            self.difficulty + 1
        } else if actual_time > TWO_WEEKS_MS * 2 {
            self.difficulty.saturating_sub(1)
        } else {
            self.difficulty
        }
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            let valid_hash = calculate_hash(
                current.index,
                current.timestamp,
                &current.previous_hash,
                current.nonce,
                &current.data,
            );

            if current.hash != valid_hash || !current.hash.starts_with(&"0".repeat(self.difficulty)) {
                return false;
            }
        }
        true
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let serialized = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        serde_json::from_str(&contents).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_validity() {
        let mut bc = Blockchain::new();
        bc.add_block("primeiro".to_string());
        bc.add_block("segundo".to_string());
        assert!(bc.is_valid());
    }

    #[test]
    fn test_reward_halving() {
        let mut bc = Blockchain::new();
        for i in 1..=(HALVING_INTERVAL * 2) {
            bc.add_block(format!("tx {}", i));
        }
        assert!(bc.block_reward < INITIAL_REWARD);
    }

    #[test]
    fn test_persistence() {
        let path = "test_blockchain.json";
        let mut bc = Blockchain::new();
        bc.add_block("persistencia".to_string());
        bc.save_to_file(path).unwrap();

        let loaded = Blockchain::load_from_file(path).unwrap();
        assert_eq!(bc.chain.len(), loaded.chain.len());
        std::fs::remove_file(path).unwrap();
    }
}
