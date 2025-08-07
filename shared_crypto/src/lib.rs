//! # ESP32 Tauri 暗号化通信ライブラリ
//!
//! ESP32とTauriアプリケーション間でAES-256-GCMを使用した
//! 軽量暗号化通信を行うためのライブラリです。
//!
//! ## 特徴
//! - AES-256-GCM暗号化
//! - SHA-256固定鍵による簡単なセットアップ
//! - ESP32とTauriの両方で使用可能
//! - Base64エンコーディングによる安全なデータ転送
//!
//! ## 使用例
//!
//! ```rust
//! use esp32_tauri_crypto::*;
//!
//! // 暗号化システムの初期化
//! let crypto = CryptoSystem::new("MY_SECRET_KEY_2025");
//!
//! // メッセージの暗号化
//! let message = "Hello ESP32!";
//! let encrypted = crypto.encrypt(message)?;
//!
//! // メッセージの復号化
//! let decrypted = crypto.decrypt(&encrypted)?;
//! ```

use serde::{Deserialize, Serialize};
use aes_gcm::{Aes256Gcm, Nonce, aead::{Aead, KeyInit}};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};
use rand_core::{OsRng, RngCore};

/// 暗号化エラーの種類
#[derive(Debug)]
pub enum CryptoError {
    /// 暗号化に失敗
    EncryptionFailed,
    /// 復号化に失敗
    DecryptionFailed,
    /// 鍵の作成に失敗
    KeyCreationFailed,
    /// Base64デコードに失敗
    Base64DecodeFailed,
    /// UTF-8デコードに失敗
    Utf8DecodeFailed,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::EncryptionFailed => write!(f, "暗号化に失敗しました"),
            CryptoError::DecryptionFailed => write!(f, "復号化に失敗しました"),
            CryptoError::KeyCreationFailed => write!(f, "暗号鍵の作成に失敗しました"),
            CryptoError::Base64DecodeFailed => write!(f, "Base64デコードに失敗しました"),
            CryptoError::Utf8DecodeFailed => write!(f, "UTF-8デコードに失敗しました"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// 暗号化されたメッセージを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Base64エンコードされた暗号文
    pub ciphertext: String,
    /// Base64エンコードされたnonce（初期化ベクトル）
    pub nonce: String,
}

/// コマンド構造体（ESP32-Tauri通信用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// アクション名
    pub action: String,
    /// オプションのデータ
    pub data: Option<String>,
}

/// レスポンス構造体（ESP32-Tauri通信用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// ステータス
    pub status: String,
    /// メッセージ内容
    pub message: String,
    /// 応答元のコマンド
    pub response_to: Option<String>,
}

/// 暗号化通信システムのメイン構造体
#[derive(Clone)]
pub struct CryptoSystem {
    /// 暗号化鍵（AES-256用の32バイト）
    key: [u8; 32],
}

impl CryptoSystem {
    /// 固定文字列から暗号化システムを作成
    /// 
    /// # 引数
    /// - `seed`: 鍵生成用の種文字列
    /// 
    /// # 例
    /// ```rust
    /// let crypto = CryptoSystem::new("ESP32_TAURI_DEMO_KEY_2025");
    /// ```
    pub fn new(seed: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let key: [u8; 32] = hasher.finalize().into();
        
        Self { key }
    }

    /// 32バイトの直接的な鍵から暗号化システムを作成
    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// 文字列を暗号化
    /// 
    /// # 引数
    /// - `plaintext`: 暗号化したい文字列
    /// 
    /// # 戻り値
    /// 暗号化されたメッセージまたはエラー
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedMessage, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| CryptoError::KeyCreationFailed)?;
        
        // ランダムなnonce生成（12バイト）
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // 暗号化実行
        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        Ok(EncryptedMessage {
            ciphertext: BASE64.encode(&ciphertext),
            nonce: BASE64.encode(&nonce_bytes),
        })
    }

    /// 暗号化されたメッセージを復号化
    /// 
    /// # 引数
    /// - `encrypted`: 暗号化されたメッセージ
    /// 
    /// # 戻り値
    /// 復号化された文字列またはエラー
    pub fn decrypt(&self, encrypted: &EncryptedMessage) -> Result<String, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| CryptoError::KeyCreationFailed)?;
        
        let nonce_bytes = BASE64.decode(&encrypted.nonce)
            .map_err(|_| CryptoError::Base64DecodeFailed)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = BASE64.decode(&encrypted.ciphertext)
            .map_err(|_| CryptoError::Base64DecodeFailed)?;
        
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| CryptoError::DecryptionFailed)?;
        
        String::from_utf8(plaintext)
            .map_err(|_| CryptoError::Utf8DecodeFailed)
    }

    /// コマンドをJSON形式で暗号化
    pub fn encrypt_command(&self, command: &Command) -> Result<EncryptedMessage, CryptoError> {
        let json = serde_json::to_string(command)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        self.encrypt(&json)
    }

    /// レスポンスをJSON形式で暗号化
    pub fn encrypt_response(&self, response: &Response) -> Result<EncryptedMessage, CryptoError> {
        let json = serde_json::to_string(response)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        self.encrypt(&json)
    }

    /// 暗号化されたメッセージからコマンドを復号化
    pub fn decrypt_to_command(&self, encrypted: &EncryptedMessage) -> Result<Command, CryptoError> {
        let json = self.decrypt(encrypted)?;
        serde_json::from_str(&json)
            .map_err(|_| CryptoError::DecryptionFailed)
    }

    /// 暗号化されたメッセージからレスポンスを復号化
    pub fn decrypt_to_response(&self, encrypted: &EncryptedMessage) -> Result<Response, CryptoError> {
        let json = self.decrypt(encrypted)?;
        serde_json::from_str(&json)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

/// 便利関数：デフォルトのシード文字列を使用して暗号化システムを作成
pub fn create_default_crypto() -> CryptoSystem {
    CryptoSystem::new("ESP32_TAURI_DEMO_KEY_2025")
}

/// 現在のタイムスタンプを取得（UNIX時間）
pub fn get_current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_round_trip() {
        let crypto = CryptoSystem::new("test_key");
        let message = "Hello, World!";
        
        let encrypted = crypto.encrypt(message).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        
        assert_eq!(message, decrypted);
    }

    #[test]
    fn test_command_encryption() {
        let crypto = CryptoSystem::new("test_key");
        let command = Command {
            action: "hello".to_string(),
            data: Some("test data".to_string()),
        };
        
        let encrypted = crypto.encrypt_command(&command).unwrap();
        let decrypted = crypto.decrypt_to_command(&encrypted).unwrap();
        
        assert_eq!(command.action, decrypted.action);
        assert_eq!(command.data, decrypted.data);
    }
}