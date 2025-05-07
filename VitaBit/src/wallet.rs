// Imports duplicados (linha antiga pode ser removida)


use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::rngs::OsRng; // ✅ Correto

use sha2::{Sha256, Digest};
use ripemd::Ripemd160;
use base58::ToBase58;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit}; // ✅ necessário para cipher.new()

use pbkdf2::pbkdf2;
use std::fs::{File, create_dir_all, read_dir};
use std::io::{Read, Write};
use std::path::Path;
use hmac::Hmac;

use serde::{Serialize, Deserialize};
use serde_json;


type HmacSha256 = Hmac<Sha256>;

/// Struct para lidar com múltiplas carteiras salvas
pub struct WalletManager;

impl WalletManager {
    pub fn save_encrypted(wallet: Wallet, name: &str, password: &str) -> std::io::Result<()> {
        wallet.save_named(name, password)
    }

    pub fn load_encrypted(name: &str, password: &str) -> Option<Wallet> {
        Wallet::load_named(name, password)
    }

    pub fn list_wallets() -> std::io::Result<Vec<String>> {
        let mut names = Vec::new();
        let paths = std::fs::read_dir("wallets")?;
        for entry in paths {
            let entry = entry?;
            if let Some(name) = entry.path().file_stem() {
                names.push(name.to_string_lossy().into_owned());
            }
        }
        Ok(names)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub private_key: SecretKey,
    pub public_key: PublicKey,
}

// ✅ Correto: geração com chave pública e privada
impl Wallet {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let (secret_key, public_key) = secp256k1::generate_keypair(&mut rng);
        Wallet {
            private_key: secret_key,
            public_key,
        }
    }

    // Gera endereço base58 do hash da chave pública (P2PKH-style)
    pub fn get_address(&self) -> String {
        let pubkey_bytes = self.public_key.serialize();
        let sha256 = Sha256::digest(&pubkey_bytes);
        let ripemd160 = Ripemd160::digest(&sha256);
        ripemd160.to_base58()
    }

    pub fn save_named(&self, name: &str, password: &str) -> std::io::Result<()> {
        create_dir_all("wallets")?;
        let path = format!("wallets/{}.wallet", name);
        self.save_to_file(&path, password)
    }

    pub fn load_named(name: &str, password: &str) -> Option<Self> {
        let path = format!("wallets/{}.wallet", name);
        Self::load_from_file(&path, password)
    }

    pub fn list_wallets() -> Vec<String> {
        let mut result = vec![];
        if let Ok(entries) = read_dir("wallets") {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem().and_then(|n| n.to_str()) {
                    result.push(name.to_string());
                }
            }
        }
        result
    }

    fn save_to_file(&self, path: &str, password: &str) -> std::io::Result<()> {
        let salt = b"vitabit-wallet-salt";
        let mut key = [0u8; 32];
        pbkdf2::<HmacSha256>(password.as_bytes(), salt, 100_000, &mut key);

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, self.private_key.secret_bytes().as_ref())
            .expect("encryption failure");

        let data = WalletData {
            encrypted_key: ciphertext,
            nonce: nonce_bytes,
            public_key: self.public_key.serialize().to_vec(),
        };

        let serialized = serde_json::to_string_pretty(&data)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn load_from_file(path: &str, password: &str) -> Option<Self> {
        if !Path::new(path).exists() {
            return None;
        }

        let mut file = File::open(path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;

        let data: WalletData = serde_json::from_str(&contents).ok()?;

        let salt = b"vitabit-wallet-salt";
        let mut key: [u8; 32] = [0u8; 32];
        pbkdf2::<HmacSha256>(password.as_bytes(), salt, 100_000, &mut key);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let nonce = Nonce::from_slice(&data.nonce);
        let decrypted = cipher.decrypt(nonce, data.encrypted_key.as_ref()).ok()?;

        let private_key = SecretKey::from_slice(&decrypted).ok()?;
        let public_key = PublicKey::from_slice(&data.public_key).ok()?;

        Some(Wallet {
            private_key,
            public_key,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct WalletData {
    encrypted_key: Vec<u8>,
    nonce: [u8; 12],
    public_key: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_wallets() {
        let wallet = Wallet::new();
        let name = "test_user";
        let password = "senha123";

        wallet.save_named(name, password).unwrap();
        let loaded = Wallet::load_named(name, password).expect("Falha ao carregar wallet");

        // Confirma que os endereços (derivados da chave pública) são iguais
        assert_eq!(wallet.get_address(), loaded.get_address());
        println!("✅ Wallet '{}' salva e carregada: {}", name, loaded.get_address());

        let list = Wallet::list_wallets();
        assert!(list.contains(&name.to_string()));
        println!("📜 Carteiras disponíveis: {:?}", list);

        std::fs::remove_file(format!("wallets/{}.wallet", name)).unwrap();
    }
}
