use rand::{rngs::OsRng, RngCore, Rng}; // <- Adicionado Rng
use std::path::Path;
use std::fs::{self, File};
use std::io::{Write, Read};
use zeroize::Zeroize;
use hex::decode;
use generic_array::GenericArray;
use aes_gcm::aead::Aead;
use chrono::Utc;

use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::ecdsa::{SigningKey, Signature, signature::Signer};

use aes_gcm::{Aes256Gcm, KeyInit, AeadCore, Key, Nonce};

use pbkdf2::pbkdf2_hmac;
use sha2::{Sha256, Digest};
use ripemd::Ripemd160;
use base58::ToBase58;
use base64::{engine::general_purpose, Engine as _};

use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
use serde::{Serialize, Deserialize};
use serde_json;

use crate::transaction::{Transaction, TxInput, TxOutput};
use crate::utxo::UTXOSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
}

impl Wallet {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng;

        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);

        let sk = SecretKey::from_slice(&secret_bytes).expect("Chave inválida");
        let pk = PublicKey::from_secret_key(&secp, &sk);

        let pub_key_bytes = pk.serialize();
        let sha256 = Sha256::digest(&pub_key_bytes);
        let ripemd160 = Ripemd160::digest(&sha256);

        let mut payload = vec![0x00];
        payload.extend(&ripemd160);
        let checksum = &Sha256::digest(&Sha256::digest(&payload))[0..4];
        payload.extend(checksum);
        let address = payload.to_base58();

        Wallet {
            private_key: hex::encode(sk.secret_bytes()),
            public_key: hex::encode(pub_key_bytes),
            address,
        }
    }

    pub fn save_encrypted(&self, name: &str, password: &str) -> std::io::Result<()> {
        let json = serde_json::to_string(self).unwrap();

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), name.as_bytes(), 100_000, &mut key);

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let ciphertext = cipher.encrypt(&nonce, json.as_bytes()).expect("Falha ao criptografar");

        let mut output = general_purpose::STANDARD.encode(&nonce);
        output.push(':');
        output.push_str(&general_purpose::STANDARD.encode(&ciphertext));

        fs::create_dir_all("wallets")?;
        let mut file = File::create(format!("wallets/{}.wallet", name))?;
        file.write_all(output.as_bytes())?;

        key.zeroize();
        Ok(())
    }

    pub fn load_encrypted(name: &str, password: &str) -> Option<Self> {
        let path = format!("wallets/{}.wallet", name);
        let mut content = String::new();
        File::open(&path).ok()?.read_to_string(&mut content).ok()?;

        let parts: Vec<&str> = content.split(':').collect();
        if parts.len() != 2 { return None; }
        let nonce = general_purpose::STANDARD.decode(parts[0]).ok()?;
        let ciphertext = general_purpose::STANDARD.decode(parts[1]).ok()?;

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), name.as_bytes(), 100_000, &mut key);

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let plain = cipher.decrypt(GenericArray::from_slice(&nonce), ciphertext.as_ref()).ok()?;
        key.zeroize();

        serde_json::from_slice(&plain).ok()
    }

   

    pub fn from_private_key(hex_priv: &str) -> Result<Self, String> {
        let secp = Secp256k1::new();

        let priv_bytes = hex::decode(hex_priv)
            .map_err(|_| "Chave privada inválida (não é hex)".to_string())?;
        let sk = SecretKey::from_slice(&priv_bytes)
            .map_err(|_| "Chave privada inválida (tamanho incorreto)".to_string())?;
        let pk = PublicKey::from_secret_key(&secp, &sk);

        let pub_key_bytes = pk.serialize();
        let sha256 = Sha256::digest(&pub_key_bytes);
        let ripemd160 = Ripemd160::digest(&sha256);

        let mut payload = vec![0x00];
        payload.extend(&ripemd160);
        let checksum = &Sha256::digest(&Sha256::digest(&payload))[0..4];
        payload.extend(checksum);
        let address = payload.to_base58();

        Ok(Wallet {
            private_key: hex_priv.to_string(),
            public_key: hex::encode(pub_key_bytes),
            address,
        })
    }

    pub fn export_private_key_encrypted(&self, senha: &str, caminho: &str) -> std::io::Result<()> {
    let mut hasher = Sha256::new();
    hasher.update(senha.as_bytes());
    let chave_derivada = hasher.finalize();

  let key = Key::<Aes256Gcm>::from_slice(&chave_derivada);
    let cipher = Aes256Gcm::new(&key);

    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, self.private_key.as_bytes())
        .expect("Erro ao criptografar chave privada");

    let dados_codificados = format!(
        "{}:{}",
        general_purpose::STANDARD.encode(nonce),
        general_purpose::STANDARD.encode(ciphertext)
    );

    let mut arquivo = File::create(caminho)?;
    arquivo.write_all(dados_codificados.as_bytes())?;

    Ok(())
}

pub fn import_private_key_encrypted(senha: &str, caminho: &str) -> Result<Wallet, String> {
    let conteudo = fs::read_to_string(caminho)
        .map_err(|e| format!("Erro ao ler arquivo: {}", e))?;

    let partes: Vec<&str> = conteudo.trim().split(':').collect();
    if partes.len() != 2 {
        return Err("Formato inválido do arquivo de backup.".to_string());
    }

    let nonce = general_purpose::STANDARD.decode(partes[0])
        .map_err(|e| format!("Nonce inválido: {}", e))?;
    let ciphertext = general_purpose::STANDARD.decode(partes[1])
        .map_err(|e| format!("Dados criptografados inválidos: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(senha.as_bytes());
    let chave_derivada = hasher.finalize();

    let key: &Key<Aes256Gcm> = Key::<Aes256Gcm>::from_slice(&chave_derivada);
    let cipher = Aes256Gcm::new(&key);

    let decrypted = cipher.decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|e| format!("Falha ao descriptografar: {}", e))?;

    let chave_privada_str = String::from_utf8(decrypted)
        .map_err(|_| "Chave privada restaurada não é UTF-8 válida.".to_string())?;

    Wallet::from_private_key(&chave_privada_str)
        .map_err(|e| format!("Erro ao restaurar carteira: {}", e))
}


    pub fn show(&self, dev_mode: bool) {
        if dev_mode {
            println!("🔐 Chave privada: {}", self.private_key);
        }
        println!("🏦 Endereço VBIT : {}", self.address);
        println!("🔓 Chave pública : {}", self.public_key);
    }

    pub fn list_wallets(password: &str) {
        let dir = Path::new("wallets");
        if !dir.exists() {
            println!("Nenhuma carteira encontrada.");
            return;
        }

        for entry in fs::read_dir(dir).unwrap() {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wallet") {
                    let file_name = path.file_stem().unwrap().to_str().unwrap();
                    print!("📁 {} → ", file_name);
                    match Wallet::load_encrypted(file_name, password) {
                        Some(wallet) => println!("Endereço: {}", wallet.address),
                        None => println!("(senha incorreta ou arquivo corrompido)"),
                    }
                }
            }
        }
    }

    pub fn create_transaction(
        &self,
        to: &str,
        amount: u64,
        utxo_set: &UTXOSet,
    ) -> Option<Transaction> {
        let available = utxo_set.find_by_address(&self.address);

        let mut total = 0;
        let mut inputs = vec![];

        let sk_bytes = hex::decode(&self.private_key).ok()?;
        let sk = SecretKey::from_slice(&sk_bytes).ok()?;

        for (txid, index, output) in &available {
            let secp = Secp256k1::new();
            let msg_hash = Sha256::digest(format!("{}{}", txid, index).as_bytes());
            let msg = Message::from_digest_slice(&msg_hash).ok()?;
            let sig = secp.sign_ecdsa(&msg, &sk);
            let sig_hex = hex::encode(sig.serialize_der());

            total += output.value;
            inputs.push(TxInput {
                txid: txid.clone(),
                index: *index,
                signature: sig_hex,
                pubkey: self.public_key.clone(),
            });

            if total >= amount {
                break;
            }
        }

        if total < amount {
            return None;
        }

        let mut outputs = vec![TxOutput {
            value: amount,
            address: to.to_string(),
            timestamp: Utc::now().timestamp(),
        }];

        if total > amount {
            outputs.push(TxOutput {
                value: total - amount,
                address: self.address.clone(),
                timestamp: Utc::now().timestamp(),
            });
        }

        Some(Transaction::new(inputs, outputs))
    }

    pub fn assinar(&self, msg_hash: &[u8]) -> Vec<u8> {
        let private_key_bytes: Vec<u8> = decode(&self.private_key).unwrap();
        let signing_key = SigningKey::from_bytes(&GenericArray::from_slice(&private_key_bytes)).unwrap();
        let signature: Signature = signing_key.sign(msg_hash);
        signature.to_bytes().to_vec()
    }
}
