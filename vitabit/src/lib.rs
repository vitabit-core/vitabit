// src/lib.rs

/// Módulo responsável pelas transações
pub mod transaction;

/// Módulo responsável por criação, criptografia e uso da carteira
pub mod wallet;

/// Módulo que define a estrutura de um bloco e mineração
pub mod block;

/// Módulo principal da cadeia de blocos (adicionar, verificar, etc)
pub mod blockchain;

/// Módulo que mantém o controle dos saldos disponíveis (UTXO)
pub mod utxo;

//rede P2P
pub mod p2p;