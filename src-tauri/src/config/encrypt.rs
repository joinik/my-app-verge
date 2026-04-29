use std::cell::Cell;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::utils::dirs::get_encryption_key;

const NONCE_LENGTH: usize = 12;

// Use task-local context so the flag follows the async task across threads
tokio::task_local! {
    static ENCRYPTION_ACTIVE: Cell<bool>;
}

/// Encrypt data
#[allow(deprecated)]
pub fn encrypt_data(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let encryption_key = get_encryption_key()?;
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);

    let cipher = Aes256Gcm::new(key);

    let mut nonce = vec![0u8; NONCE_LENGTH];
    getrandom::fill(&mut nonce)?;

    // Encrypt data
    let ciphertext = cipher
        .encrypt(nonce.as_slice().into(), data.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Concatenate nonce and ciphertext and encode them in Base64
    let mut combined = nonce;
    combined.extend(ciphertext);
    Ok(STANDARD.encode(combined))
}

/// 使用 AES-256-GCM 解密数据
#[allow(deprecated)]
pub fn decrypt_data(encrypted: &str) -> Result<String, Box<dyn std::error::Error>> {
    let encryption_key = get_encryption_key()?;
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);

    let cipher = Aes256Gcm::new(key);

    // Decode from base64
    let data = STANDARD.decode(encrypted)?;
    if data.len() < NONCE_LENGTH {
        return Err("Invalid encrypted data".into());
    }

    // Separate nonce and ciphertext
    let (nonce, ciphertext) = data.split_at(NONCE_LENGTH);

    // Decrypt data
    let plaintext = cipher
        .decrypt(nonce.into(), ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| e.into())
}

/// 序列化并加密数据的自定义序列化器
///
/// 该函数用于在 serde 序列化过程中自动加密字段数据。
/// 当加密功能激活时，它会先将数据序列化为 JSON 字符串，然后加密该字符串；
/// 否则直接进行正常的序列化。
///
/// # 参数
/// * `value` - 要序列化的值的引用
/// * `serializer` - serde 序列化器
///
/// # 返回
/// * `Result<S::Ok, S::Error>` - 序列化结果或序列化错误
///
/// # 泛型参数
/// * `T` - 要序列化的类型，必须实现 Serialize trait
/// * `S` - 序列化器类型
///
/// # 工作流程
/// 1. 检查加密是否激活
/// 2. 如果激活：将值序列化为 JSON → 加密 JSON 字符串 → 返回加密后的字符串
/// 3. 如果未激活：直接正常序列化值
///
pub fn serialize_encrypted<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,  // T 必须实现 Serialize trait
    S: Serializer, // S 必须实现 Serializer trait
{
    // 检查加密功能是否激活（通过 tokio task-local 变量）
    if is_encryption_actice() {
        // 第一步：将值序列化为 JSON 字符串
        // 使用 serde_json 将任意类型 T 转换为字符串表示
        let json = serde_json::to_string(value).map_err(serde::ser::Error::custom)?;

        // 第二步：加密 JSON 字符串
        // encrypt_data 返回 Base64 编码的加密字符串
        let encrypted = encrypt_data(&json).map_err(serde::ser::Error::custom)?;

        // 第三步：将加密后的字符串序列化
        // 因为加密结果是字符串，所以使用 serialize_str
        serializer.serialize_str(&encrypted)
    } else {
        // 加密功能未激活，直接进行正常的序列化
        // 让值使用其默认的序列化实现
        value.serialize(serializer)
    }
}

/// 反序列化加密数据的自定义反序列化器
///
/// 该函数用于在 serde 反序列化过程中自动解密加密的字段。
/// 当加密功能激活时，它会先解密数据，然后将解密后的字符串反序列化为目标类型；
/// 否则直接进行正常的反序列化。
///
/// # 参数
/// * `deserializer` - serde 反序列化器
///
/// # 返回
/// * `Result<T, D::Error>` - 反序列化后的值或反序列化错误
///
/// # 泛型参数
/// * `T` - 目标类型，必须实现 Deserialize 和 Default trait
/// * `D` - 反序列化器类型
pub fn deserialize_encrypted<'a, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: for<'de> Deserialize<'de> + Default, // T 必须能在任意生命周期反序列化，且有默认值
    D: Deserializer<'a>,                    // D 必须实现 Deserializer trait
{
    // 检查加密功能是否激活（通过 tokio task-local 变量）
    if is_encryption_actice() {
        // 先尝试将输入反序列化为 Option<String>
        // 这样处理字段可能不存在或为 null 的情况
        let encrypted_opt: Option<String> = Option::deserialize(deserializer)?;

        match encrypted_opt {
            // 如果有加密数据且不为空字符串
            Some(encrypted) if !encrypted.is_empty() => {
                // 第一步：解密数据，将 serde 错误转换为反序列化器错误类型
                let decrypted_string = decrypt_data(&encrypted).map_err(serde::de::Error::custom)?;

                // 第二步：将解密后的 JSON 字符串反序列化为目标类型 T
                // 这里假设加密前数据是以 JSON 格式序列化的
                serde_json::from_str(&decrypted_string).map_err(serde::de::Error::custom)
            }
            // 如果字段不存在、为 null 或为空字符串，返回类型的默认值
            _ => Ok(T::default()),
        }
    } else {
        // 加密功能未激活，直接进行正常的反序列化
        T::deserialize(deserializer)
    }
}

fn is_encryption_actice() -> bool {
    ENCRYPTION_ACTIVE.try_with(|c| c.get()).unwrap_or(false)
}

pub async fn with_encryption<F, Fut, R>(f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    ENCRYPTION_ACTIVE.scope(Cell::new(true), f()).await
}
