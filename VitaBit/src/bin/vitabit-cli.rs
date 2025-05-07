use vitabit::wallet::{Wallet, WalletManager};
use std::env;
use std::process;

fn print_usage() {
    println!("Vitabit CLI - Comandos disponíveis:");
    println!("  create-wallet <nome>");
    println!("  list-wallets");
    println!("  load-wallet <nome>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = args[1].as_str();
    match command {
        "create-wallet" => {
            if args.len() != 3 {
                println!("Uso: create-wallet <nome>");
                process::exit(1);
            }
            let name = &args[2];
            println!("Senha para proteger a carteira:");
            let password = rpassword::prompt_password("Senha: ").unwrap();
            let wallet = Wallet::new();
            if let Err(e) = WalletManager::save_encrypted(wallet.clone(), name, &password) {
                eprintln!("❌ Erro ao salvar carteira: {}", e);
                process::exit(1);
            }
            println!("✅ Nova carteira criada: {}", wallet.get_address());
        }

        "list-wallets" => {
            match WalletManager::list_wallets() {
                Ok(names) => println!("📜 Carteiras disponíveis: {:?}", names),
                Err(e) => {
                    eprintln!("❌ Erro ao listar carteiras: {}", e);
                    process::exit(1);
                }
            }
        }

        "load-wallet" => {
            if args.len() != 3 {
                println!("Uso: load-wallet <nome>");
                process::exit(1);
            }
            let name = &args[2];
            println!("Digite a senha da carteira:");
            let password = rpassword::prompt_password("Senha: ").unwrap();

            match WalletManager::load_encrypted(name, &password) {
                Some(wallet) => println!("✅ Carteira carregada: {}", wallet.get_address()),
                None => {
                    eprintln!("❌ Falha ao carregar carteira. Verifique o nome ou a senha.");
                    process::exit(1);
                }
            }
        }

        _ => {
            print_usage();
            process::exit(1);
        }
    }
}
