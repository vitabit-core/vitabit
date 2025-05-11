// src/utxo.rs

use crate::blockchain::Blockchain;
use crate::transaction::{Transaction, TxInput, TxOutput};
use crate::block::Block; 

use std::collections::HashMap;

const SEGUNDOS_100_ANOS: i64 = 100 * 365 * 24 * 60 * 60; // 100 anos, sem bissexto

/// Conjunto de UTXOs não gastos
#[derive(Debug, Clone)]
pub struct UTXOSet {
    pub utxos: HashMap<String, Vec<(usize, TxOutput)>>, // txid → (índice, output)
}

impl UTXOSet {


    pub fn remove(&mut self, txid: &str, index: usize) {
        if let Some(outputs) = self.utxos.get_mut(txid) {
            outputs.retain(|(i, _)| *i != index);
            if outputs.is_empty() {
                self.utxos.remove(txid);
            }
        }
    }


    /// Retorna UTXOs inativos por mais de 100 anos
    pub fn utxos_inativos(&self, _blockchain: &Blockchain, tempo_atual: i64) -> Vec<(String, usize, TxOutput)> {
        let mut inativos = Vec::new();

        for (txid, outputs) in &self.utxos {
            for (index, output) in outputs {
                if tempo_atual - output.timestamp >= SEGUNDOS_100_ANOS {
                    inativos.push((txid.clone(), *index, output.clone()));
                }
            }
        }

        inativos
    }

    /// Cria UTXOSet completo da blockchain
    pub fn from_blockchain(blockchain: &Blockchain) -> Self {
        let mut spent: HashMap<String, Vec<usize>> = HashMap::new();
        let mut utxos: HashMap<String, Vec<(usize, TxOutput)>> = HashMap::new();

        for block in &blockchain.chain {
            let txs: Vec<Transaction> = serde_json::from_str(&block.data).unwrap_or_default();

            for tx in &txs {
                // Marca entradas como gastas
                for input in &tx.inputs {
                    spent.entry(input.txid.clone())
                         .or_default()
                         .push(input.index);
                }

                // Registra outputs não gastos
                for (index, output) in tx.outputs.iter().enumerate() {
                    let txid = tx.id.clone();
                    let is_spent = spent.get(&txid)
                                        .map_or(false, |idxs| idxs.contains(&index));
                    if !is_spent {
                        utxos.entry(txid.clone())
                             .or_default()
                             .push((index, output.clone()));
                    }
                }
            }
        }

        UTXOSet { utxos }
    }

    /// Filtra UTXOs por endereço
    pub fn find_by_address(&self, address: &str) -> Vec<(String, usize, TxOutput)> {
        let mut results = vec![];

        for (txid, outputs) in &self.utxos {
            for (index, output) in outputs {
                if output.address == address {
                    results.push((txid.clone(), *index, output.clone()));
                }
            }
        }

        results
    }

    /// Saldo total de um endereço
    pub fn balance(&self, address: &str) -> u64 {
        self.find_by_address(address)
            .iter()
            .map(|(_, _, output)| output.value)
            .sum()
    }

    /// Construtor vazio
    pub fn new() -> Self {
        UTXOSet {
            utxos: HashMap::new(),
        }
    }

    /// Constrói UTXOSet a partir de parte da cadeia
    pub fn from_chain_segment(chain: &[Block]) -> Self {
        let mut utxos: HashMap<String, Vec<(usize, TxOutput)>> = HashMap::new();

        for block in chain {
            let txs: Vec<Transaction> = match serde_json::from_str(&block.data) {
                Ok(t) => t,
                Err(_) => continue,
            };

            for tx in &txs {
                for (i, output) in tx.outputs.iter().enumerate() {
                    utxos.entry(tx.id.clone())
                         .or_default()
                         .push((i, output.clone()));
                }

                for input in &tx.inputs {
                    if let Some(outputs) = utxos.get_mut(&input.txid) {
                        outputs.retain(|(i, _)| *i != input.index);
                        if outputs.is_empty() {
                            utxos.remove(&input.txid);
                        }
                    }
                }
            }
        }

        UTXOSet { utxos }
    }

    /// Calcula o total de VBITs não gastos (em circulação)
    pub fn total_em_circulacao(&self) -> u64 {
        self.utxos
            .values()
            .flat_map(|outputs| outputs.iter().map(|(_, out)| out.value))
            .sum()
    }
}
