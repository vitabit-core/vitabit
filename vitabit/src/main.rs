mod block;
mod p2p;
mod wallet;
mod blockchain;
mod transaction;
mod utxo;

use crate::wallet::Wallet;
use crate::blockchain::Blockchain;
use crate::utxo::UTXOSet;
use crate::transaction::{TxInput, TxOutput, Transaction};
use crate::block::Block;

use std::net::TcpStream;
use rpassword::prompt_password;
use std::io::{self, Write};
use sha2::Digest;
use hex;
use chrono::Utc;
use serde_json;
use p2p::P2PServer;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let dev_mode = true; // mude para true se quiser ver a chave privada/ false para esconder
    println!("=== VitaBit CLI ===");
    

    let nome = "usuario";
    let senha = prompt_password("\u{1f510} Digite a senha da carteira: ").unwrap();

    let wallet = if let Some(w) = Wallet::load_encrypted(nome, &senha) {
        println!("\u{2705} Carteira carregada.");
        w
    } else {
        println!("⚠️ Nenhuma carteira encontrada. Criando nova...");
        let w = Wallet::new();
        w.save_encrypted(nome, &senha).unwrap();
        w
    };

    wallet.show(dev_mode);
    println!("👷 Recompensa do bloco gênesis atribuída a: {}", wallet.address);

    let caminho_bc = "blockchain.json";
    let mut bc = if let Some(bc) = Blockchain::carregar_de_arquivo(caminho_bc) {
        println!("\u{2705} Blockchain carregada do disco.");
        bc
    } else {
        println!("⚠️ Nenhuma blockchain encontrada. Criando nova...");
        Blockchain::new()
    };

    bc.salvar_em_arquivo(caminho_bc).expect("Erro ao salvar blockchain.");
    let mut utxos = UTXOSet::from_blockchain(&bc);

    let genesis_reward = Blockchain::calcular_recompensa(0);
    let coinbase_tx = Transaction::new_coinbase(&wallet.address, genesis_reward);
    let bloco_genesis = bc.add_block(
        serde_json::to_string(&vec![coinbase_tx.clone()]).unwrap(),
        &wallet.address,
        &mut utxos,
    );

    let bloco_json = match serde_json::to_string(&bloco_genesis) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("\u{274c} Erro ao serializar bloco gênesis: {}", e);
            return;
        }
    };

    let blockchain_arc = Arc::new(Mutex::new(bc.clone()));
    let servidor_p2p = P2PServer::new();

    let servidor_clone = servidor_p2p.clone();
    let blockchain_clone = Arc::clone(&blockchain_arc);

    thread::spawn(move || {
        servidor_clone.start(6010, blockchain_clone);
    });

    servidor_p2p.iter_peers(|peer| {
        if let Ok(mut stream) = TcpStream::connect(peer) {
            servidor_p2p.enviar_bloco(&mut stream, &bloco_json);
            println!("\u{1f4e4} Bloco gênesis enviado para peer {}", peer);
        } else {
            eprintln!("⚠️ Falha ao conectar ao peer {}", peer);
        }
    });

    println!("\u{2705} Bloco gênesis criado: {}", bloco_genesis.index);
    println!("\u{1f4b0} Saldo atual: {} VBIT", utxos.balance(&wallet.address));
    println!("\u{2705} Blockchain válida? {}", bc.is_valid());

    loop {
        println!("\nEscolha uma opção:");
        println!("1. Consultar saldo");
        println!("2. Enviar transação");
        println!("3. Verificar blocos");
        println!("4. Listar peers conectados");
        println!("5. Conectar a um peer remoto");
        println!("7. Exportar chave privada (backup)");
        println!("8. Restaurar carteira de backup");
        println!("6. Sair");

        let mut escolha = String::new();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut escolha).unwrap();

        match escolha.trim() {
            "1" => consultar_saldo(&utxos, &wallet),
            "2" => enviar_transacao(&mut blockchain_arc.lock().unwrap(), &mut utxos, &wallet, &servidor_p2p),
            "3" => verificar_blocos(&blockchain_arc.lock().unwrap()),
            "4" => servidor_p2p.listar_peers(),
            "5" => {
                let mut endereco = String::new();
                println!("Digite o endereço do peer (ex: 127.0.0.1:6010):");
                io::stdin().read_line(&mut endereco).unwrap();
                servidor_p2p.conectar_a_peer(endereco.trim());
            }
            
            "7" => {
                
    println!("Digite o caminho do arquivo para salvar (ex: backup.key):");
    let mut caminho = String::new();
    io::stdin().read_line(&mut caminho).unwrap();

    let caminho = caminho.trim();
    match wallet.export_private_key_encrypted(&senha, caminho) {
        Ok(_) => println!("🔐 Backup da chave privada exportado para '{}'", caminho),
        Err(e) => eprintln!("❌ Falha ao exportar chave privada: {}", e),
        
    }
    
            }

            "8" => {
    println!("Digite o caminho do backup (ex: backup.key):");
    let mut caminho = String::new();
    io::stdin().read_line(&mut caminho).unwrap();
    let caminho = caminho.trim();

    match Wallet::import_private_key_encrypted(&senha, caminho) {
        Ok(w) => {
            println!("✅ Carteira restaurada com sucesso!");
            w.show(false);
        }
        Err(e) => {
            eprintln!("❌ Falha ao restaurar carteira: {}", e);
        }
    }
}


            "6" => {
                println!("Saindo...");
                break;
            }
            _ => println!("Opção inválida!"),
        }
    }
}

fn consultar_saldo(utxos: &UTXOSet, wallet: &Wallet) {
    let saldo = utxos.balance(&wallet.address);
    println!("\u{1f4b0} Saldo atual: {} VBIT", saldo);
}

fn enviar_transacao(bc: &mut Blockchain, utxos: &mut UTXOSet, wallet: &Wallet, servidor_p2p: &P2PServer) {
    let mut destino = String::new();
    let mut valor_str = String::new();

    println!("Digite o endereço de destino:");
    std::io::stdin().read_line(&mut destino).unwrap();
    let destino = destino.trim();

    println!("Digite o valor a enviar (em VBIT):");
    std::io::stdin().read_line(&mut valor_str).unwrap();
    let valor: u64 = match valor_str.trim().parse() {
        Ok(v) => v,
        Err(_) => {
            println!("⚠️ Valor inválido.");
            return;
        }
    };

    let meus_utxos = utxos.find_by_address(&wallet.address);
    let mut acumulado = 0;
    let mut inputs = vec![];

    for (txid, index, output) in meus_utxos {
        acumulado += output.value;
        let msg = format!("{}{}", txid, index);
        let msg_hash = sha2::Sha256::digest(msg.as_bytes());
        let assinatura = wallet.assinar(&msg_hash);
        let assinatura_hex = hex::encode(assinatura);

        inputs.push(TxInput {
            txid,
            index,
            signature: assinatura_hex,
            pubkey: wallet.public_key.clone(),
        });

        if acumulado >= valor {
            break;
        }
    }

    if acumulado < valor {
        println!("⚠️ Saldo insuficiente.");
        return;
    }

    let mut outputs = vec![TxOutput {
        value: valor,
        address: destino.to_string(),
        timestamp: Utc::now().timestamp(),
    }];

    if acumulado > valor {
        outputs.push(TxOutput {
            value: acumulado - valor,
            address: wallet.address.clone(),
            timestamp: Utc::now().timestamp(),
        });
    }

    let tx = Transaction::new(inputs, outputs);
    let txs = vec![tx.clone()];
    let txs_json = serde_json::to_string(&txs).unwrap();

    let novo_bloco = bc.add_block(txs_json, &wallet.address, utxos);
    *utxos = UTXOSet::from_blockchain(bc);

    println!("\u{2705} Transação incluída no bloco {}", novo_bloco.index);

    for peer in servidor_p2p.get_peers().iter() {
        servidor_p2p.enviar_transacao(peer, &tx);
    }

    let bloco_json = serde_json::to_string(&novo_bloco).unwrap();
    for peer in servidor_p2p.get_peers().iter() {
        if let Ok(mut stream) = TcpStream::connect(peer) {
            servidor_p2p.enviar_bloco(&mut stream, &bloco_json);
        }
    }
}

fn verificar_blocos(bc: &Blockchain) {
    let blocos = bc.get_blocks();
    println!("\u{1f4e6} Blocos na blockchain:");
    for bloco in blocos {
        println!("\n\u{1f539} Bloco #{}", bloco.index);
        println!("   Data: {}", bloco.timestamp);
        println!("   Transações: {}", bloco.transactions.len());
    }
}
