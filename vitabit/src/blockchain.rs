const AJUSTE_INTERVALO: usize = 2016;
const TEMPO_ESPERADO: i64 = 1209600; // 2 semanas em segundos

// src/blockchain.rs


use crate::transaction::{TxInput, TxOutput, Transaction};
use crate::utxo::UTXOSet;
use crate::block::Block; 

use serde::{Serialize, Deserialize};
use chrono::Utc;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub total_em_circulacao: u64, // Novo campo para rastrear total em circulação
}

impl Blockchain {

    pub fn salvar_em_arquivo(&self, caminho: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.chain)?;
        let mut file = File::create(caminho)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn carregar_de_arquivo(caminho: &str) -> Option<Blockchain> {
        let mut file = match File::open(caminho) {
            Ok(f) => f,
            Err(_) => return None,
        };

        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_err() {
            return None;
        }

        match serde_json::from_str::<Vec<Block>>(&contents) {
            Ok(chain) => Some(Blockchain { chain, difficulty: 4, total_em_circulacao: 0 }),

            Err(_) => None,
        }
    }


    pub fn verify_block(&self, block_index: usize, utxo_set: &UTXOSet) -> bool {
        if block_index == 0 || block_index >= self.chain.len() {
            return false;
        }

        let block = &self.chain[block_index];
        let previous = &self.chain[block_index - 1];

        if block.previous_hash != previous.hash {
            return false;
        }

        if block.hash != block.calculate_hash() {
            return false;
        }

        let txs: Vec<Transaction> = match serde_json::from_str(&block.data) {
            Ok(t) => t,
            Err(_) => return false,
        };

        for tx in txs.iter().skip(1) {
            if !self.verify_transaction(tx, utxo_set) {
                return false;
            }
        }

        true
    }

    pub fn verify_transaction(&self, tx: &Transaction, utxo_set: &UTXOSet) -> bool {
        for input in &tx.inputs {
            let referenced = match utxo_set.utxos.get(&input.txid) {
                Some(r) => r,
                None => return false,
            };

            let output = match referenced.iter().find(|(i, _)| *i == input.index) {
                Some((_, out)) => out,
                None => return false,
            };

            if !input.is_valid(output) {
                return false;
            }
        }
        true
    }

    pub fn verify_chain(&self) -> bool {
        let mut utxo = UTXOSet::new();

        for i in 1..self.chain.len() {
            if !self.verify_block(i, &utxo) {
                println!("❌ Bloco #{} inválido", i);
                return false;
            }

            utxo = UTXOSet::from_chain_segment(&self.chain[..=i]);
        }

        println!("✅ Blockchain 100% válida");
        true
    }

    pub fn new() -> Self {
        let genesis = Block::genesis();
        Blockchain {
            chain: vec![genesis],
            difficulty: 4,
            total_em_circulacao: 0,
        }
    }

    pub fn latest_hash(&self) -> String {
        self.chain.last().unwrap().hash.clone()
    }

    pub fn height(&self) -> u64 {
        self.chain.len() as u64
    }

    pub fn calcular_recompensa(altura_bloco: u64) -> u64 {
        let recompensa_inicial = 50 * 100_000_000; // 50 VBIT
        let halvings = altura_bloco / 210_000;
        let recompensa = recompensa_inicial >> halvings;
        if recompensa > 0 { recompensa } else { 0 }
    }

    pub fn calcular_recompensa_extra(
        utxo_set: &mut UTXOSet,
        blockchain: &Blockchain,
        tempo_atual: i64,
        vbit_em_circulacao: u64
    ) -> u64 {
        let inativos = utxo_set.utxos_inativos(blockchain, tempo_atual);
        let mut total_reabsorvido = 0;

        for (txid, index, output) in inativos {
            utxo_set.remove(&txid, index);
            total_reabsorvido += output.value;
        }

        let limite = 21_000_000 * 100_000_000;
        if vbit_em_circulacao + total_reabsorvido > limite {
            0
        } else if vbit_em_circulacao + total_reabsorvido > limite {
            limite - vbit_em_circulacao
        } else {
            total_reabsorvido
        }
    }

    pub fn add_block(&mut self, data: String, miner_address: &str, utxo_set: &mut UTXOSet) -> Block {
        let tempo_atual = Utc::now().timestamp();

        // Calcula recompensa extra a partir de UTXOs inativos
        let extra_reward = Blockchain::calcular_recompensa_extra(
            utxo_set,
            self,
            tempo_atual,
            self.total_em_circulacao,
        );

        let last_block = self.chain.last().unwrap();
        let index = last_block.index + 1;
        let previous_hash = last_block.hash.clone();

        let base_reward = Blockchain::calcular_recompensa(index as u64);
        let total_reward = base_reward + extra_reward;

        let mut reward_tx = Transaction::new_coinbase(miner_address, total_reward);
        let mut txs: Vec<Transaction> = serde_json::from_str(&data).unwrap_or_default();
        txs.insert(0, reward_tx);

        let txs_json = serde_json::to_string(&txs).unwrap();
        let new_block = Block::new(index, previous_hash, txs_json, total_reward);
        self.chain.push(new_block.clone());

        self.total_em_circulacao += total_reward;
        self.ajustar_dificuldade();

        new_block
    }

    pub fn ajustar_dificuldade(&mut self) {
        let altura = self.chain.len();

        if altura % AJUSTE_INTERVALO != 0 || altura <= AJUSTE_INTERVALO {
            return;
        }

        let bloco_atual = &self.chain[altura - 1];
        let bloco_ajuste = &self.chain[altura - AJUSTE_INTERVALO];
        let tempo_real = bloco_atual.timestamp - bloco_ajuste.timestamp;

        if tempo_real < TEMPO_ESPERADO / 2 {
            self.difficulty += 1;
            println!("⏫ Aumentando dificuldade para {}", self.difficulty);
        } else if tempo_real > TEMPO_ESPERADO * 2 {
            self.difficulty = self.difficulty.saturating_sub(1);
            println!("⏬ Reduzindo dificuldade para {}", self.difficulty);
        } else {
            println!("🔁 Mantendo dificuldade em {}", self.difficulty);
        }
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.hash != current.calculate_hash() {
                return false;
            }

            if current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }

    pub fn get_blocks(&self) -> &Vec<Block> {
        &self.chain
    }
}
