use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use keyring::Entry;
use rand::RngCore;
use std::env;

pub struct VaultManager {
    master_key: [u8; 32],
}

impl VaultManager {
    pub fn new() -> Result<Self, String> {
        // We use a dedicated entry for the master key in the OS keyring
        let entry = Entry::new("SelfHOSTed", "master_key").map_err(|e| e.to_string())?;

        // 1. Check if key exists in keyring
        if let Ok(encoded_key) = entry.get_password() {
            if let Ok(decoded) = BASE64.decode(&encoded_key) {
                if decoded.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&decoded);
                    return Ok(Self { master_key: key });
                }
            }
        }

        // 2. Check headless fallback
        if let Ok(env_key) = env::var("SELFHOSTED_MASTER_KEY") {
            if let Ok(decoded) = BASE64.decode(&env_key) {
                if decoded.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&decoded);
                    return Ok(Self { master_key: key });
                }
            }
        }

        // 3. Generate new master key
        let mut new_key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut new_key);
        let encoded_new_key = BASE64.encode(&new_key);

        // 4. Try storing in OS keyring
        if let Err(e) = entry.set_password(&encoded_new_key) {
            // Secure Fail: Do NOT fallback to plaintext DB storage
            return Err(format!(
                "Failed to access OS Keyring ({}). Secure OS credential storage is unavailable in this environment. Configure the SELFHOSTED_MASTER_KEY environment variable with a base64-encoded 32-byte key to enable encrypted secrets storage.",
                e
            ));
        }

        Ok(Self { master_key: new_key })
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<String, String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        
        // Generate a random 96-bit nonce per encryption
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Payload format: [12 bytes nonce][... ciphertext]
        let mut payload = nonce.to_vec();
        payload.extend_from_slice(&ciphertext);
        
        Ok(BASE64.encode(payload))
    }

    pub fn decrypt_string(&self, encrypted_payload: &str) -> Result<String, String> {
        let decoded = BASE64.decode(encrypted_payload)
            .map_err(|_| "Failed to decode base64 payload")?;

        if decoded.len() < 12 {
            return Err("Invalid encrypted payload length".into());
        }

        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Nonce::from_slice(&decoded[0..12]);
        let ciphertext = &decoded[12..];

        let plaintext_bytes = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext_bytes)
            .map_err(|_| "Decrypted data is not valid UTF-8".into())
    }

    pub fn generate_secure_password(&self) -> String {
        // Generate a 24-character cryptographically secure random password
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789!@#$%^&*()-_=+";
        let mut password = String::with_capacity(24);
        let mut rng = rand::thread_rng();
        for _ in 0..24 {
            let idx = (rng.next_u32() as usize) % CHARSET.len();
            password.push(CHARSET[idx] as char);
        }
        password
    }
}
