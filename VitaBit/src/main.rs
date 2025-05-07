// vitabit/src/main.rs

mod block;
mod blockchain;
mod wallet;

use vitabit::wallet::{Wallet};
use blockchain::Blockchain;

fn main() {
    let mut bc = Blockchain::new();

    bc.add_block("primeiro bloco real".to_string());
    bc.add_block("segundo bloco real".to_string());

    match bc.save_to_file("vitabit_chain.json") {
        Ok(_) => println!("✅ Blockchain salva em 'vitabit_chain.json'"),
        Err(e) => eprintln!("❌ Erro ao salvar blockchain: {}", e),
    }

    match Blockchain::load_from_file("vitabit_chain.json") {
        Some(loaded) => {
            println!("📦 Blockchain carregada com {} blocos", loaded.chain.len());
            for block in loaded.chain {
                println!("🔗 Bloco #{}", block.index);
                println!("⏱️ Timestamp: {}", block.timestamp);
                println!("🧱 Hash: {}", block.hash);
                println!("🔒 Previous Hash: {}", block.previous_hash);
                println!("📜 Dados: {}\n", block.data);
            }
        },
        None => println!("⚠️ Falha ao carregar blockchain"),
    }
}
