# 🎯 ESP32-Tauri暗号化通信システム 完全チュートリアル

Rust初心者向けの詳細な解説とステップバイステップガイドです。

## 📋 目次

1. [システム概要](#システム概要)
2. [環境構築](#環境構築) 
3. [プロジェクト作成](#プロジェクト作成)
4. [ライブラリの理解](#ライブラリの理解)
5. [実践的な使い方](#実践的な使い方)
6. [デプロイとビルド](#デプロイとビルド)
7. [応用とカスタマイズ](#応用とカスタマイズ)

## 🎯 システム概要

### アーキテクチャ図
```
┌─────────────────┐    USB Serial    ┌─────────────────┐
│     ESP32       │ ←──暗号化通信──→ │   Tauri App     │
│  (Rust + IDF)   │                  │ (Rust + React)  │
└─────────────────┘                  └─────────────────┘
         │                                     │
         ├── AES-256-GCM暗号化                │
         ├── JSON形式通信                     │  
         └── 定期ハートビート                 └── GUI表示・操作
```

### データフロー
```
1. ESP32 → JSON作成 → AES暗号化 → Base64 → Serial送信
2. Tauri ← JSON解析 ← AES復号化 ← Base64 ← Serial受信
```

## 🛠️ 環境構築

### Step 1: Rustのインストール

```bash
# 1. Rustをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 環境変数を更新
source ~/.cargo/env

# 3. インストール確認
rustc --version
cargo --version
```

### Step 2: ESP32開発環境

```bash
# 1. ESP32用ツールチェインを追加
rustup target add xtensa-esp32s3-espidf

# 2. espflashをインストール（ESP32フラッシュツール）
cargo install espflash

# 3. ESP-IDFをインストール（公式ガイドに従う）
# https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/
```

### Step 3: Node.js環境（Tauri用）

```bash
# 1. Node.js（18以上）をインストール
# macOS
brew install node

# または nvm使用
nvm install 18
nvm use 18

# 2. Tauri CLIをインストール
npm install -g @tauri-apps/cli

# 3. 確認
node --version
npm --version
tauri --version
```

## 📂 プロジェクト作成

### Step 1: ディレクトリ構造の作成

```bash
# プロジェクトルートを作成
mkdir esp32-tauri-crypto && cd esp32-tauri-crypto

# サブプロジェクトを作成
mkdir shared_crypto backend gui
```

### Step 2: 共通ライブラリの作成

```bash
cd shared_crypto
cargo init --lib

# Cargo.tomlを編集（上記のライブラリ設定を参考）
```

### Step 3: ESP32プロジェクトの作成

```bash
cd ../backend
cargo init

# ESP32用の設定ファイルを作成
touch sdkconfig.defaults
```

**sdkconfig.defaults**:
```ini
CONFIG_ESP_MAIN_TASK_STACK_SIZE=32768
CONFIG_ESP_SYSTEM_EVENT_TASK_STACK_SIZE=4096
CONFIG_ESP_TASK_WDT_EN=n
CONFIG_ESP_TASK_WDT_INIT=n
```

### Step 4: Tauriプロジェクトの作成

```bash
cd ../gui
npm create tauri-app@latest . -- --template react-ts
npm install
```

## 🔬 ライブラリの理解

### 共通暗号化ライブラリ（shared_crypto）

#### 主要な構造体

```rust
// 暗号化されたデータを表現
pub struct EncryptedMessage {
    pub ciphertext: String,  // Base64暗号文
    pub nonce: String,       // Base64 nonce（初期化ベクトル）
}

// ESP32-Tauri間のコマンド
pub struct Command {
    pub action: String,           // "hello", "ping"など
    pub data: Option<String>,     // オプションのペイロード
}

// ESP32からのレスポンス
pub struct Response {
    pub status: String,               // "ok", "error"など
    pub message: String,              // メッセージ内容
    pub timestamp: u64,               // UNIX時間
    pub response_to: Option<String>,  // 元のコマンド名
}
```

#### CryptoSystemクラス

```rust
impl CryptoSystem {
    // 1. 初期化
    pub fn new(seed: &str) -> Self {
        // SHA-256でseedから32バイト鍵を生成
    }
    
    // 2. 暗号化
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedMessage, CryptoError> {
        // AES-256-GCMで暗号化
        // ランダムnonce生成
        // Base64でエンコード
    }
    
    // 3. 復号化
    pub fn decrypt(&self, encrypted: &EncryptedMessage) -> Result<String, CryptoError> {
        // Base64でデコード
        // AES-256-GCMで復号化
        // UTF-8文字列に変換
    }
}
```

### ESP32ライブラリ（backend）

#### 通信ハンドラー

```rust
pub struct ESP32CommunicationHandler {
    crypto: CryptoSystem,  // 暗号化システム
}

impl ESP32CommunicationHandler {
    // 暗号化レスポンス送信
    pub fn send_encrypted_response(&self, status: &str, message: &str, response_to: Option<String>) {
        // 1. Response構造体作成
        // 2. JSON文字列に変換
        // 3. 暗号化
        // 4. println!でシリアル出力
    }
    
    // コマンド処理
    pub fn handle_command(&self, command: &Command) {
        match command.action.as_str() {
            "hello" => { /* Hello処理 */ },
            "ping" => { /* Ping処理 */ },
            // ...
        }
    }
}
```

## 📋 実践的な使い方

### ケース1: 新しいコマンドの追加

**要求**: "get_temperature" コマンドを追加してESP32の温度を取得

#### ESP32側の実装

```rust
// backend/src/lib.rs
impl ESP32CommunicationHandler {
    pub fn handle_command(&self, command: &Command) {
        match command.action.as_str() {
            // ... 既存のコマンド
            "get_temperature" => {
                // 仮の温度データ
                let temp = "25.6°C";
                self.send_encrypted_response(
                    "temperature_response", 
                    temp, 
                    Some("get_temperature".to_string())
                );
            },
        }
    }
    
    // 温度送信専用メソッド
    pub fn send_temperature(&self) {
        let temperature = self.read_temperature(); // 仮実装
        self.send_encrypted_response(
            "temperature", 
            &format!("Current temp: {}°C", temperature),
            None
        );
    }
    
    fn read_temperature(&self) -> f32 {
        // 実際のセンサー読み取り実装
        25.6 // ダミーデータ
    }
}
```

#### Tauri側の実装

```rust
// gui/src-tauri/src/main.rs

#[tauri::command]
fn request_temperature(port_name_state: State<'_, Arc<Mutex<PortNameState>>>) -> Result<String, String> {
    // 温度要求コマンドを送信
    send_command_internal(port_name_state, "get_temperature", None)
}
```

#### フロントエンド（React）

```tsx
// gui/src/App.tsx
const requestTemperature = async () => {
    try {
        const result = await invoke<string>("request_temperature");
        console.log("Temperature requested:", result);
    } catch (error) {
        alert(`温度要求エラー: ${error}`);
    }
};

// JSX内にボタン追加
<button onClick={requestTemperature}>
    🌡️ 温度取得
</button>
```

### ケース2: カスタム暗号化鍵の使用

```rust
// 両方のプロジェクトで同じカスタム鍵を使用

// backend/src/lib.rs
impl ESP32CommunicationHandler {
    pub fn with_custom_key(custom_seed: &str) -> Self {
        Self {
            crypto: CryptoSystem::new(custom_seed),
        }
    }
}

// main.rs内
let handler = ESP32CommunicationHandler::with_custom_key("MySecretKey2025");

// Tauri側でも同じ鍵を使用
// gui/src-tauri/src/main.rs
.manage(Arc::new(Mutex::new(SimpleCryptoState {
    shared_key: generate_custom_key("MySecretKey2025"),
    is_ready: true,
})))
```

## 🚀 デプロイとビルド

### Development（開発環境）

```bash
# 1. ESP32をビルドしてフラッシュ
cd backend
cargo build
espflash flash --port /dev/cu.usbserial-11230 --monitor target/xtensa-esp32s3-espidf/debug/backend

# 2. Tauriアプリを開発モードで起動
cd ../gui  
npm run tauri dev
```

### Production（本番環境）

```bash
# 1. ESP32 リリースビルド
cd backend
cargo build --release
espflash flash --port /dev/cu.usbserial-11230 target/xtensa-esp32s3-espidf/release/backend

# 2. Tauri リリースビルド
cd ../gui
npm run tauri build

# 生成物: gui/src-tauri/target/release/bundle/
```

### クロスプラットフォームビルド

```bash
# macOS向け
npm run tauri build

# Windows向け（macOSから）
npm run tauri build -- --target x86_64-pc-windows-msvc

# Linux向け（macOSから）  
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## 🎨 応用とカスタマイズ

### 高度なカスタマイズ例

#### 1. センサーデータの周期送信

```rust
// ESP32側: 定期的なセンサーデータ送信
pub fn run_sensor_loop(interval_ms: u32, sensor_types: Vec<&str>) {
    let handler = ESP32CommunicationHandler::new();
    let mut counter = 0u32;
    
    loop {
        counter += 1;
        
        for sensor_type in &sensor_types {
            match *sensor_type {
                "temperature" if counter % 10 == 0 => {
                    handler.send_temperature();
                }
                "humidity" if counter % 15 == 0 => {
                    handler.send_humidity();
                }
                "pressure" if counter % 20 == 0 => {
                    handler.send_pressure(); 
                }
                _ => {}
            }
        }
        
        FreeRtos::delay_ms(interval_ms);
    }
}

// main.rsで使用
run_sensor_loop(1000, vec!["temperature", "humidity", "pressure"]);
```

#### 2. 設定ファイル管理

```rust
// 設定構造体
#[derive(Serialize, Deserialize)]
pub struct ESP32Config {
    pub device_id: String,
    pub send_interval_ms: u32,
    pub enabled_sensors: Vec<String>,
    pub encryption_key: String,
}

impl ESP32Config {
    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        // ファイルから設定を読み込み
    }
    
    pub fn save_to_file(&self, path: &str) -> Result<(), ConfigError> {
        // ファイルに設定を保存
    }
}
```

#### 3. エラーハンドリングの改善

```rust
// カスタムエラー型
#[derive(Debug)]
pub enum ESP32Error {
    CommunicationError(String),
    SensorError(String),
    ConfigurationError(String),
    CryptoError(esp32_tauri_crypto::CryptoError),
}

impl From<esp32_tauri_crypto::CryptoError> for ESP32Error {
    fn from(err: esp32_tauri_crypto::CryptoError) -> Self {
        ESP32Error::CryptoError(err)
    }
}
```

### パフォーマンス最適化

#### メモリ使用量の監視

```rust
// ESP32でのヒープ使用量監視
fn log_heap_usage() {
    use esp_idf_svc::sys::*;
    
    unsafe {
        let free_heap = heap_caps_get_free_size(MALLOC_CAP_8BIT);
        let min_free = heap_caps_get_minimum_free_size(MALLOC_CAP_8BIT);
        println!("Free heap: {}, Min free: {}", free_heap, min_free);
    }
}
```

#### 暗号化処理の最適化

```rust
// バッチ処理による効率化
impl CryptoSystem {
    pub fn encrypt_batch(&self, messages: Vec<&str>) -> Result<Vec<EncryptedMessage>, CryptoError> {
        messages.into_iter()
            .map(|msg| self.encrypt(msg))
            .collect()
    }
}
```

## 🔧 デバッグとテスト

### ユニットテスト

```rust
// shared_crypto/src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let crypto = CryptoSystem::new("test_key");
        let original = "Hello, ESP32!";
        
        let encrypted = crypto.encrypt(original).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_command_serialization() {
        let command = Command {
            action: "test".to_string(),
            data: Some("test data".to_string()),
        };
        
        let crypto = CryptoSystem::new("test_key");
        let encrypted = crypto.encrypt_command(&command).unwrap();
        let decrypted = crypto.decrypt_to_command(&encrypted).unwrap();
        
        assert_eq!(command.action, decrypted.action);
        assert_eq!(command.data, decrypted.data);
    }
}
```

### 統合テスト

```bash
# テスト実行
cd shared_crypto
cargo test

cd ../backend  
cargo test

cd ../gui/src-tauri
cargo test
```

### ログとモニタリング

```rust
// ESP32側のログ設定
use log::{info, warn, error};

impl ESP32CommunicationHandler {
    pub fn send_encrypted_response(&self, status: &str, message: &str, response_to: Option<String>) {
        info!("Sending encrypted response: status={}, message_len={}", status, message.len());
        
        // 暗号化処理...
        match encrypted_result {
            Ok(encrypted) => {
                info!("Encryption successful");
                // 送信処理...
            },
            Err(e) => {
                error!("Encryption failed: {:?}", e);
            }
        }
    }
}
```

## 📊 メトリクスと監視

### 通信統計

```rust
#[derive(Debug)]
pub struct CommunicationStats {
    pub messages_sent: u32,
    pub messages_received: u32,
    pub encryption_errors: u32,
    pub last_communication: u64,
}

impl ESP32CommunicationHandler {
    pub fn get_stats(&self) -> &CommunicationStats {
        &self.stats
    }
    
    pub fn reset_stats(&mut self) {
        self.stats = CommunicationStats::default();
    }
}
```

このチュートリアルに従えば、RustとTauriが初めての方でも暗号化通信システムを理解し、カスタマイズできるようになります。さらに詳しい質問があれば、いつでもお聞かせください！