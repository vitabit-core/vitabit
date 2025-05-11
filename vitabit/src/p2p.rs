use std::collections::HashSet;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{Read, Write};

use crate::blockchain::Blockchain;
use crate::transaction::Transaction;

/// Estrutura do servidor P2P
#[derive(Clone)]
pub struct P2PServer {
    peers: Arc<Mutex<HashSet<String>>>, // lista de peers conectados
}

impl P2PServer {
    pub fn get_peers(&self) -> Vec<String> {
        self.peers.lock().unwrap().iter().cloned().collect() // ou outro tipo de retorno adequado
    }

    /// Envia um bloco JSON para um peer via stream TCP
    pub fn enviar_bloco(&self, stream: &mut TcpStream, bloco_json: &str) {
        let msg = format!("BLOCK:{}", bloco_json);
        let _ = stream.write_all(msg.as_bytes());
    }

    /// Cria um novo servidor P2P
    pub fn new() -> Self {
        P2PServer {
            peers: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Itera sobre os peers conhecidos e aplica uma função
    pub fn iter_peers<F>(&self, mut f: F)
    where
        F: FnMut(&String),
    {
        let peers = self.peers.lock().unwrap();
        for addr in peers.iter() {
            f(addr);
        }
    }

    /// Inicia o servidor P2P em uma porta específica
    pub fn start(&self, porta: u16, blockchain: Arc<Mutex<Blockchain>>) {
        let listener = TcpListener::bind(("0.0.0.0", porta)).expect("Erro ao iniciar servidor P2P");
        println!("🌐 Servidor P2P escutando na porta {}", porta);

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let blockchain_clone = Arc::clone(&blockchain);
                thread::spawn(move || {
                    P2PServer::handle_connection(&mut stream, blockchain_clone);
                });
            }
        }
    }

    /// Trata cada conexão de peer
    fn handle_connection(stream: &mut TcpStream, blockchain: Arc<Mutex<Blockchain>>) {
        let mut buffer = vec![0; 1024];
        match stream.read(&mut buffer) {
            Ok(_) => {
                let mensagem = String::from_utf8_lossy(&buffer);
                if mensagem.contains("TRANSACTION:") {
                    let json = mensagem.replace("TRANSACTION:", "").trim().to_string();
                    match serde_json::from_str::<Transaction>(&json) {
                        Ok(tx) => {
                            println!("📨 Transação recebida via P2P: {:?}", tx);
                            // TODO: adicionar à fila de transações pendentes
                        }
                        Err(e) => println!("❌ Erro ao desserializar transação: {}", e),
                    }
                } else if mensagem.contains("BLOCK:") {
                    let json = mensagem.replace("BLOCK:", "").trim().to_string();
                    // TODO: validar e adicionar bloco à blockchain
                    println!("📦 Bloco recebido via P2P: {}", json);
                }
            }
            Err(e) => {
                println!("❌ Erro ao ler do stream: {}", e);
            }
        }
    }

    /// Envia uma transação para um peer remoto
    pub fn enviar_transacao(&self, endereco: &str, tx: &Transaction) {
        if let Ok(mut stream) = TcpStream::connect(endereco) {
            let json = serde_json::to_string(tx).unwrap();
            let mensagem = format!("TRANSACTION:{}", json);
            let _ = stream.write_all(mensagem.as_bytes());
            println!("📤 Transação enviada para {}", endereco);
        } else {
            println!("⚠️ Falha ao conectar com peer {}", endereco);
        }
    }

    /// Lista os peers conectados
    pub fn listar_peers(&self) {
        let peers = self.peers.lock().unwrap();
        println!("🔗 Peers conectados:");
        for peer in peers.iter() {
            println!("- {}", peer);
        }
    }

    /// Conecta-se a um novo peer e o adiciona à lista
    pub fn conectar_a_peer(&self, endereco: &str) {
        let mut peers = self.peers.lock().unwrap();
        if peers.insert(endereco.to_string()) {
            println!("✅ Conectado ao novo peer: {}", endereco);
        } else {
            println!("ℹ️ Já conectado ao peer: {}", endereco);
        }
    }
}
