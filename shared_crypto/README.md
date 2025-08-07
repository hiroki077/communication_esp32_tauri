# ESP32 Tauri 暗号化ライブラリ

ESP32とTauriアプリケーション間の軽量AES-256-GCM暗号化通信を提供するライブラリです。

## ✨ 特徴

- 🔐 **AES-256-GCM暗号化**: 高セキュリティ
- 🚀 **軽量設計**: ESP32でも動作可能  
- 🔄 **双方向通信**: コマンド・レスポンス対応
- 📦 **シンプルAPI**: 簡単な実装
- 🎯 **エラーハンドリング**: Rust標準エラー処理

## 🚀 クイックスタート

### Cargo.tomlに追加

```toml
[dependencies]
esp32_tauri_crypto = { path = "path/to/shared_crypto" }
```

### 基本的な使用例

```rust
use esp32_tauri_crypto::{CryptoSystem, Command, Response};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 暗号化システムを作成
    let crypto = CryptoSystem::new("MY_SECRET_KEY_2025");
    
    // メッセージを暗号化
    let message = "Hello, secure world!";
    let encrypted = crypto.encrypt(message)?;
    
    // メッセージを復号化
    let decrypted = crypto.decrypt(&encrypted)?;
    println!("Decrypted: {}", decrypted);
    
    Ok(())
}
```

## 📚 API ドキュメント

### CryptoSystem

```rust
impl CryptoSystem {
    /// シード文字列から暗号化システムを作成
    pub fn new(seed: &str) -> Self
    
    /// 32バイト鍵から直接作成
    pub fn from_key(key: [u8; 32]) -> Self
    
    /// 文字列を暗号化
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedMessage, CryptoError>
    
    /// 暗号化メッセージを復号化
    pub fn decrypt(&self, encrypted: &EncryptedMessage) -> Result<String, CryptoError>
    
    /// コマンドを暗号化
    pub fn encrypt_command(&self, command: &Command) -> Result<EncryptedMessage, CryptoError>
    
    /// レスポンスを暗号化
    pub fn encrypt_response(&self, response: &Response) -> Result<EncryptedMessage, CryptoError>
    
    /// 暗号化メッセージからコマンドを復号化
    pub fn decrypt_to_command(&self, encrypted: &EncryptedMessage) -> Result<Command, CryptoError>
    
    /// 暗号化メッセージからレスポンスを復号化
    pub fn decrypt_to_response(&self, encrypted: &EncryptedMessage) -> Result<Response, CryptoError>
}
```

### 主要な構造体

#### EncryptedMessage
```rust
pub struct EncryptedMessage {
    pub ciphertext: String,  // Base64暗号文
    pub nonce: String,       // Base64 nonce
}
```

#### Command
```rust
pub struct Command {
    pub action: String,           // アクション名
    pub data: Option<String>,     // オプションデータ
}
```

#### Response
```rust
pub struct Response {
    pub status: String,               // ステータス
    pub message: String,              // メッセージ
    pub timestamp: u64,               // タイムスタンプ
    pub response_to: Option<String>,  // 応答元コマンド
}
```

#### CryptoError
```rust
pub enum CryptoError {
    EncryptionFailed,      // 暗号化失敗
    DecryptionFailed,      // 復号化失敗
    KeyCreationFailed,     // 鍵作成失敗
    Base64DecodeFailed,    // Base64デコード失敗
    Utf8DecodeFailed,      // UTF-8デコード失敗
}
```

## 💡 使用例

### ESP32での暗号化送信

```rust
use esp32_tauri_crypto::{CryptoSystem, Response, get_current_timestamp};

let crypto = CryptoSystem::new("ESP32_SECURE_KEY");

let response = Response {
    status: "ok".to_string(),
    message: "Hello from ESP32!".to_string(),
    timestamp: get_current_timestamp(),
    response_to: Some("hello".to_string()),
};

let encrypted = crypto.encrypt_response(&response)?;
let json = serde_json::to_string(&encrypted)?;
println!("{}", json); // シリアル送信
```

### Tauriでの暗号化受信

```rust
use esp32_tauri_crypto::{CryptoSystem, EncryptedMessage};

#[tauri::command]
fn decrypt_received_message(encrypted_json: String) -> Result<String, String> {
    let crypto = CryptoSystem::new("ESP32_SECURE_KEY");
    let encrypted: EncryptedMessage = serde_json::from_str(&encrypted_json)
        .map_err(|e| e.to_string())?;
    let response = crypto.decrypt_to_response(&encrypted)
        .map_err(|e| e.to_string())?;
    Ok(response.message)
}
```

### バッチ処理

```rust
let messages = vec!["message1", "message2", "message3"];
let encrypted_messages: Result<Vec<_>, _> = messages
    .into_iter()
    .map(|msg| crypto.encrypt(msg))
    .collect();
```

## 🛡️ セキュリティ仕様

| 項目 | 仕様 |
|------|------|
| 暗号化アルゴリズム | AES-256-GCM |
| 鍵長 | 256ビット (32バイト) |
| 鍵生成 | SHA-256ハッシュ |
| Nonce | 96ビット (12バイト) ランダム |
| エンコーディング | Base64 |

## ⚠️ セキュリティ注意事項

1. **固定鍵**: 本ライブラリはデモ用途で固定鍵を使用
2. **実用環境**: 適切な鍵管理システムを実装すること
3. **鍵の保護**: ソースコードに鍵を埋め込まない
4. **定期更新**: 定期的な鍵ローテーションを推奨

## 🧪 テスト

```bash
cargo test
```

テストカバレッジ:
- 暗号化・復号化のラウンドトリップ
- コマンド・レスポンスのシリアライゼーション
- エラーハンドリング

## 📖 詳細ドキュメント

詳細な使用方法は以下を参照：
- [完全チュートリアル](../TUTORIAL.md)
- [プロジェクトREADME](../README.md)

## 🤝 ライセンス

MIT License