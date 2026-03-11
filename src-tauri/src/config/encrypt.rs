use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_ENGINE};

use futures::future::err;
use rand::Rng;

/// 使用 AES-256-GCM 加密数据
pub fn encrypt(data: &str, key: &str) -> Result<String> {
    // 生成随机密钥
    let key_bytes = key.as_bytes();
    let cipher = Aes256Gcm::new(key_bytes.into());

    // 生成随机 Nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密数据
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // 编码 Base64
    let mut result = BASE64_ENGINE.encode(&nonce_bytes);
    result.push(':');
    result.push_str(&BASE64_ENGINE.encode(&ciphertext));
    Ok(result)
}

/// 使用 AES-256-GCM 解密数据
pub fn decrypt(encrypted: &str, key: &str) -> Result<String> {
    // 解码 Base64
    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 2 {
        // 立即返回错误
        anyhow::bail!("Invalid encrypted format");
    }
    let nonce_bytes = BASE64_ENGINE
        .decode(parts[0])
        .map_err(|e| anyhow::anyhow!("Invalid nonce format: {}", e))?;
    let ciphertext = BASE64_ENGINE
        .decode(parts[1])
        .map_err(|e| anyhow::anyhow!("Failed to decode ciphertext:{}", e))?;
    if nonce_bytes.len() != 12 {
        anyhow::bail!("Invalid nonce length");
    }
    let key_bytes = key.as_bytes();
    let cipher = Aes256Gcm::new(key_bytes.into());
    let nonce = Nonce::from_slice(&nonce_bytes);
    // 解密数据
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("Failed to convert plaintext to string: {}", e))
}

/// 生成随机 32 字节密钥（Base64 编码）
pub fn generate_key(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut rng = rand::thread_rng();
    let mut result = String::with_capacity(length);
    for _ in 0..length {
        let idx = rng.gen_range(0..CHARSET.len());
        result.push(CHARSET[idx] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = "Hello, World!";
        let key = "test-key-32-characters-long!!";

        let encrypted = encrypt(data, key).unwrap();
        let decrypted = decrypt(&encrypted, key).unwrap();

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_generate_key() {
        let key = generate_key(32);
        assert_eq!(key.len(), 32);

        let key2 = generate_key(32);
        assert_ne!(key, key2);
    }
}
