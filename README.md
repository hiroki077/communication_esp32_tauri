# ESP32 ⇄ Tauri シリアル通信ライブラリ

ESP32とTauriアプリケーション間で簡単にシリアル通信を行うためのRustライブラリです。

## 🎯 特徴

- **シンプル**: 平文JSON形式での双方向通信
- **堅牢**: エラーハンドリングとタイムアウト処理
- **拡張可能**: 暗号化やカスタムプロトコルに対応
- **初心者向け**: 詳細なドキュメントと使用例

## 📦 プロジェクト構成

```
esp32-tauri-serial/
├── backend/          # ESP32ファームウェア（組み込み側）
│   ├── src/
│   │   ├── main.rs   # メイン実行ファイル
│   │   └── lib.rs    # 通信ライブラリ
│   ├── Cargo.toml    # ESP32依存関係
│   └── sdkconfig.defaults # ESP32設定
├── gui/              # Tauriデスクトップアプリ（PC側）
│   ├── src-tauri/    # Rust backend
│   ├── src/          # React frontend  
│   └── package.json  # Node.js依存関係
├── shared_crypto/    # 共通通信ライブラリ
│   ├── src/lib.rs    # 共通データ構造
│   └── Cargo.toml    # ライブラリ依存関係
└── README.md         # このファイル
```

## 🚀 クイックスタート

### 1. 必要な環境

#### ESP32開発環境
```bash
# Rust ESP32ツールチェーンをインストール
curl -LO https://github.com/esp-rs/rust-build/releases/download/v1.77.2.0/install-rust-toolchain.sh
chmod +x install-rust-toolchain.sh
./install-rust-toolchain.sh

# ESP Flashツール
cargo install cargo-espflash
```

#### Tauri開発環境
```bash
# Node.js (推奨: v18以上)
# システムに応じてインストール

# Rust (安定版)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Tauriコマンド
cargo install tauri-cli
```

### 2. ハードウェア接続

- ESP32をUSBケーブルでPCに接続
- シリアルポートを確認: 
  ```bash
  ls /dev/cu.usbserial*  # macOS
  ls /dev/ttyUSB*        # Linux
  ls                     # Windows (COMポート)
  ```

### 3. ESP32ファームウェアのビルド・書き込み

```bash
cd backend

# ビルド
cargo build --release

# ESP32に書き込み（ポートは環境に合わせて変更）
cargo espflash flash --target xtensa-esp32s3-espidf --port /dev/cu.usbserial-11230 --baud 115200 --release
```

### 4. Tauriアプリケーションの起動

```bash
cd gui

# 依存関係のインストール
npm install

# 開発モードで起動
npm run tauri dev
```

### 5. 通信テスト

1. Tauriアプリが起動したら、シリアルポートを選択
2. "Start Serial Listener"を押して接続開始  
3. "Hello"ボタンを押してテスト
4. 以下の応答が表示されれば成功:
   ```
   ✅ 👋 Hello from ESP32!
   ```

## 📚 詳細ガイド（初心者向け）

### Rust とは？

**Rust**は安全性とパフォーマンスを重視したプログラミング言語です：
- **メモリ安全**: バッファオーバーフローなどの脆弱性を防ぐ
- **高性能**: C/C++並みの実行速度
- **組み込み**: ESP32のような小さなデバイスでも動作

### Tauri とは？

**Tauri**はRustでデスクトップアプリを作るフレームワークです：
- **軽量**: Electronより高速・省メモリ
- **安全**: Rustの安全性を活用
- **クロスプラットフォーム**: Windows、Mac、Linux対応

## 🛠️ ESP32側の実装

### 基本的な使用方法

```rust
use esp32_tauri_crypto::{Command, Response};
use serde_json;

// 1. レスポンス送信関数
fn send_response(status: &str, message: &str, response_to: Option<&str>) {
    let response = Response {
        status: status.to_string(),
        message: message.to_string(),
        response_to: response_to.map(|s| s.to_string()),
    };
    
    if let Ok(json) = serde_json::to_string(&response) {
        println!("{}", json);  // シリアル出力（Tauriが受信）
    }
}

// 2. コマンド処理
fn process_command(command: &Command) {
    match command.action.as_str() {
        "hello" => {
            send_response("hello_response", "👋 Hello from ESP32!", Some("hello"));
        }
        "get_temperature" => {
            // 温度センサーの値を取得（例）
            let temp = 25.5; // あなたのセンサー読み取り処理
            let message = format!("Temperature: {}°C", temp);
            send_response("temperature_data", &message, Some("get_temperature"));
        }
        "set_led" => {
            // LEDを制御（例）
            if let Some(data) = &command.data {
                if data == "on" {
                    // GPIO制御でLEDをON
                    send_response("led_status", "LED turned ON", Some("set_led"));
                } else {
                    // GPIO制御でLEDをOFF
                    send_response("led_status", "LED turned OFF", Some("set_led"));
                }
            }
        }
        _ => {
            send_response("error", "Unknown command", Some(&command.action));
        }
    }
}
```

### メインループ（main.rs）

```rust
use esp_idf_svc::sys::link_patches;
use std::thread;
use backend::run_communication_loop;

fn main() {
    link_patches();

    // 通信ループを開始（無限ループ）
    thread::Builder::new()
        .name("esp32_serial_communication".into())
        .stack_size(16 * 1024)  // 16KBスタック
        .spawn(|| run_communication_loop(0))  // 引数は使用されない
        .unwrap()
        .join()
        .unwrap();
}
```

## 💻 Tauri側の実装

### バックエンド（Rust）

```rust
use tauri::{command, State};
use std::sync::{Arc, Mutex};
use serialport;

// シリアルポート管理用の型
type SharedSerialPort = Arc<Mutex<Option<Box<dyn serialport::SerialPort>>>>;

#[tauri::command]
fn send_command(
    serial_port_state: State<SharedSerialPort>,
    action: String, 
    data: Option<String>
) -> Result<String, String> {
    let command = Command { action: action.clone(), data };
    let json_command = serde_json::to_string(&command)
        .map_err(|e| format!("JSON serialization error: {}", e))?;
    
    let mut serial_lock = serial_port_state.lock().unwrap();
    if let Some(port) = serial_lock.as_mut() {
        let command_with_newline = json_command.clone() + "\\n";
        port.write_all(command_with_newline.as_bytes())
            .map_err(|e| format!("Failed to write: {}", e))?;
        port.flush()
            .map_err(|e| format!("Failed to flush: {}", e))?;
        
        println!("📤 Sent command: {}", json_command);
        Ok(format!("Command '{}' sent successfully", action))
    } else {
        Err("Serial port not connected. Please start serial listener first.".to_string())
    }
}

#[tauri::command]
fn start_serial_listener(
    app: tauri::AppHandle, 
    msg_state: State<Arc<Mutex<MessageState>>>, 
    port_name_state: State<Arc<Mutex<PortNameState>>>,
    serial_port_state: State<SharedSerialPort>,
    port_name: String
) -> Result<(), String> {
    // 実装詳細は省略（実際のコードを参照）
    Ok(())
}
```

### フロントエンド（TypeScript/React）

```tsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

function App() {
  const [message, setMessage] = useState<string>('');
  const [portName, setPortName] = useState<string>('/dev/cu.usbserial-11230');
  const [isConnected, setIsConnected] = useState<boolean>(false);

  // ESP32にコマンドを送信
  const sendHello = async () => {
    try {
      const result = await invoke('send_command', {
        action: 'hello',
        data: null
      });
      console.log('送信結果:', result);
    } catch (error) {
      console.error('送信エラー:', error);
    }
  };

  // シリアル接続開始
  const startSerial = async () => {
    try {
      await invoke('start_serial_listener', {
        portName: portName
      });
      setIsConnected(true);
      console.log('シリアル接続開始');
    } catch (error) {
      console.error('接続エラー:', error);
    }
  };

  // カスタムコマンドの例
  const getTemperature = async () => {
    try {
      await invoke('send_command', {
        action: 'get_temperature',
        data: null
      });
    } catch (error) {
      console.error('温度取得エラー:', error);
    }
  };

  const setLED = async (state: 'on' | 'off') => {
    try {
      await invoke('send_command', {
        action: 'set_led',
        data: state
      });
    } catch (error) {
      console.error('LED制御エラー:', error);
    }
  };

  return (
    <div className="container">
      <h1>ESP32 ⇄ Tauri Serial Communication</h1>
      
      {/* シリアルポート設定 */}
      <div className="serial-setup">
        <input
          type="text"
          value={portName}
          onChange={(e) => setPortName(e.target.value)}
          placeholder="シリアルポート名"
        />
        <button onClick={startSerial} disabled={isConnected}>
          {isConnected ? '接続済み' : 'シリアル接続開始'}
        </button>
      </div>

      {/* コマンドボタン */}
      <div className="commands">
        <button onClick={sendHello} disabled={!isConnected}>
          👋 Hello
        </button>
        <button onClick={getTemperature} disabled={!isConnected}>
          🌡️ 温度取得
        </button>
        <button onClick={() => setLED('on')} disabled={!isConnected}>
          💡 LED ON
        </button>
        <button onClick={() => setLED('off')} disabled={!isConnected}>
          💡 LED OFF
        </button>
      </div>

      {/* メッセージ表示 */}
      <div className="message">
        <h3>ESP32からのメッセージ:</h3>
        <p>{message}</p>
      </div>
    </div>
  );
}

export default App;
```

## 🔧 カスタマイズ例

### 新しいコマンドの追加

#### 1. ESP32側（backend/src/lib.rs）

```rust
fn process_command(command: &Command) {
    match command.action.as_str() {
        // 既存のコマンド...
        
        "read_sensor" => {
            // センサーデータを読み取る例
            let sensor_value = read_analog_pin(34); // GPIO34から読み取り
            let message = format!("Sensor value: {}", sensor_value);
            send_response("sensor_data", &message, Some("read_sensor"));
        }
        
        "control_servo" => {
            // サーボモーターを制御する例
            if let Some(angle_str) = &command.data {
                if let Ok(angle) = angle_str.parse::<i32>() {
                    set_servo_angle(angle); // あなたの実装
                    send_response("servo_status", 
                                &format!("Servo set to {}°", angle), 
                                Some("control_servo"));
                } else {
                    send_response("error", "Invalid angle", Some("control_servo"));
                }
            }
        }
        
        "get_wifi_status" => {
            // Wi-Fi状態を取得する例
            let status = check_wifi_status(); // あなたの実装
            send_response("wifi_status", &status, Some("get_wifi_status"));
        }
        
        _ => {
            send_response("error", "Unknown command", Some(&command.action));
        }
    }
}

// ヘルパー関数の例
fn read_analog_pin(pin: u32) -> u32 {
    // ADC読み取りの実装
    // 実際のハードウェアアクセスコード
    0 // プレースホルダー
}

fn set_servo_angle(angle: i32) {
    // PWM制御でサーボモーターを動かす
    // 実際のハードウェアアクセスコード
}

fn check_wifi_status() -> String {
    // Wi-Fi接続状態をチェック
    "Connected to MyNetwork".to_string() // プレースホルダー
}
```

#### 2. Tauri側コマンド追加

```rust
#[tauri::command]
fn read_sensor(serial_port_state: State<SharedSerialPort>) -> Result<String, String> {
    send_command_internal(serial_port_state, "read_sensor".to_string(), None)
}

#[tauri::command]
fn control_servo(serial_port_state: State<SharedSerialPort>, angle: i32) -> Result<String, String> {
    send_command_internal(serial_port_state, "control_servo".to_string(), Some(angle.to_string()))
}

#[tauri::command]
fn get_wifi_status(serial_port_state: State<SharedSerialPort>) -> Result<String, String> {
    send_command_internal(serial_port_state, "get_wifi_status".to_string(), None)
}

// main関数でコマンドを登録
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // 既存のハンドラー...
            read_sensor,
            control_servo,
            get_wifi_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri");
}
```

#### 3. フロントエンド追加

```tsx
// 新しいコマンド用の関数
const readSensor = async () => {
  try {
    await invoke('read_sensor');
  } catch (error) {
    console.error('センサー読み取りエラー:', error);
  }
};

const controlServo = async (angle: number) => {
  try {
    await invoke('control_servo', { angle });
  } catch (error) {
    console.error('サーボ制御エラー:', error);
  }
};

const getWifiStatus = async () => {
  try {
    await invoke('get_wifi_status');
  } catch (error) {
    console.error('Wi-Fi状態取得エラー:', error);
  }
};

// UIに追加するボタン
<div className="custom-commands">
  <button onClick={readSensor} disabled={!isConnected}>
    📊 センサー読み取り
  </button>
  <button onClick={() => controlServo(90)} disabled={!isConnected}>
    🔄 サーボ 90°
  </button>
  <button onClick={getWifiStatus} disabled={!isConnected}>
    📶 Wi-Fi状態
  </button>
</div>
```

## 🔒 暗号化の追加（オプション）

このライブラリには暗号化機能も含まれています：

### ESP32側での暗号化

```rust
use esp32_tauri_crypto::{CryptoSystem, EncryptedMessage};

let crypto = CryptoSystem::new("your_secret_key_2025");

// 暗号化して送信
fn send_encrypted_response(status: &str, message: &str, response_to: Option<&str>) {
    let response = Response {
        status: status.to_string(),
        message: message.to_string(),
        response_to: response_to.map(|s| s.to_string()),
    };
    
    if let Ok(encrypted) = crypto.encrypt_response(&response) {
        if let Ok(encrypted_json) = serde_json::to_string(&encrypted) {
            println!("{}", encrypted_json);
        }
    }
}
```

### Tauri側での復号化

```rust
#[tauri::command]
fn decrypt_message(encrypted: EncryptedMessage) -> Result<String, String> {
    let crypto = esp32_tauri_crypto::create_default_crypto();
    crypto.decrypt(&encrypted).map_err(|e| e.to_string())
}
```

## 🐛 トラブルシューティング

### よくある問題と解決方法

#### 1. シリアルポートが見つからない
```bash
# ポートの確認
ls /dev/cu.usbserial*  # macOS
ls /dev/ttyUSB*        # Linux

# Windowsの場合
# デバイスマネージャーでCOMポートを確認

# 権限の確認 (Linux)
sudo usermod -a -G dialout $USER  # ログアウト/ログインが必要
# または
sudo chmod 666 /dev/ttyUSB0
```

#### 2. ESP32の書き込みエラー
```bash
# ESP32をブートモードにする
# ブートボタンを押しながらENボタンを押して離し、その後ブートボタンを離す

# 他のシリアル接続を閉じる
pkill screen
pkill minicom

# 書き込み再実行
cargo espflash flash --target xtensa-esp32s3-espidf --port /dev/cu.usbserial-11230 --baud 115200 --release
```

#### 3. Tauriビルドエラー
```bash
# Node.jsの依存関係の問題
rm -rf node_modules package-lock.json
npm install

# Rustキャッシュのクリア
cargo clean
cd gui/src-tauri
cargo clean

# システム依存関係の確認（Linux）
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

#### 4. シリアル通信エラー
```bash
# デバイスが使用中
Error: Device or resource busy

# 解決方法：
lsof /dev/cu.usbserial-11230  # 使用中のプロセスを確認
kill -9 <PID>                 # プロセスを終了

# または再起動
sudo reboot
```

#### 5. JSONパースエラー
```bash
# ESP32側のログを確認
espflash monitor /dev/cu.usbserial-11230

# Tauri側のログを確認（開発者ツールのコンソール）
# ブラウザのF12キーを押してConsoleタブを確認
```

### デバッグ方法

#### 1. ESP32のログ確認
```bash
# リアルタイムモニタリング
espflash monitor /dev/cu.usbserial-11230

# または
screen /dev/cu.usbserial-11230 115200
# 終了: Ctrl+A → K → Y
```

#### 2. Tauriのデバッグ
```bash
# 開発モードで詳細ログ
RUST_LOG=debug npm run tauri dev

# または環境変数を設定
export RUST_LOG=debug
npm run tauri dev
```

#### 3. 通信の直接確認
```bash
# シリアルポートに直接コマンド送信（テスト用）
echo '{"action":"hello","data":null}' > /dev/cu.usbserial-11230

# ESP32からの応答を確認
cat /dev/cu.usbserial-11230
```

## 📖 学習リソース

### Rust入門
- [The Rust Programming Language（日本語版）](https://doc.rust-jp.rs/book-ja/) - Rust公式ドキュメント日本語版
- [Rust By Example（日本語版）](https://doc.rust-jp.rs/rust-by-example-ja/) - 実例で学ぶRust
- [Tour of Rust（日本語）](https://tourofrust.com/ja/) - インタラクティブなRustチュートリアル

### ESP32 + Rust
- [The Rust on ESP Book](https://esp-rs.github.io/book/) - ESP32 Rust開発の総合ガイド
- [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/) - ESP32公式ドキュメント
- [ESP32-Rust Examples](https://github.com/esp-rs/esp-idf-hal) - サンプルコード集

### Tauri
- [Tauri Documentation](https://tauri.app/v1/guides/) - Tauri公式ドキュメント
- [Tauri Examples](https://github.com/tauri-apps/examples) - 様々なサンプルアプリケーション
- [React + Tauri Tutorial](https://tauri.app/v1/guides/getting-started/setup/html-css-js) - フロントエンド統合ガイド

### シリアル通信
- [serialport-rs](https://github.com/serialport/serialport-rs) - Rustシリアル通信ライブラリ
- [ESP32 UART Communication](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/peripherals/uart.html) - ESP32 UART公式ドキュメント

## 📋 データ形式仕様

### コマンド形式（PC → ESP32）
```json
{
  "action": "command_name",      // 必須: コマンド名（文字列）
  "data": "optional_data"        // オプション: 追加データ（文字列またはnull）
}
```

**例:**
```json
{"action": "hello", "data": null}
{"action": "set_led", "data": "on"}  
{"action": "control_servo", "data": "90"}
```

### レスポンス形式（ESP32 → PC）
```json
{
  "status": "response_status",   // 必須: レスポンスのステータス
  "message": "response_message", // 必須: レスポンスメッセージ
  "response_to": "command_name"  // オプション: 元のコマンド名
}
```

**例:**
```json
{"status": "hello_response", "message": "👋 Hello from ESP32!", "response_to": "hello"}
{"status": "led_status", "message": "LED turned ON", "response_to": "set_led"}
{"status": "error", "message": "Unknown command", "response_to": "invalid_cmd"}
```

### 通信プロトコル

1. **フォーマット**: UTF-8 JSON
2. **区切り文字**: 改行文字（`\\n`）
3. **ボーレート**: 115,200 bps
4. **データビット**: 8
5. **パリティ**: なし
6. **ストップビット**: 1
7. **フロー制御**: なし

## 🔧 設定ファイル

### ESP32設定（sdkconfig.defaults）
```ini
# メインタスクスタックサイズ
CONFIG_ESP_MAIN_TASK_STACK_SIZE=32768

# USB Serial/JTAG設定
CONFIG_ESP_CONSOLE_USB_SERIAL_JTAG=y
CONFIG_ESP_CONSOLE_USB_SERIAL_JTAG_ENABLED=y
CONFIG_TINYUSB_CDC_ENABLED=y

# タスクウォッチドッグ無効化
CONFIG_ESP_TASK_WDT_EN=n
```

### Cargo.toml（ESP32側）
```toml
[package]
name = "backend"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "backend"
harness = false

[dependencies]
esp-idf-svc = { version = "0.51", features = ["alloc", "std", "binstart"] }
esp32_tauri_crypto = { path = "../shared_crypto", features = ["esp32"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
```

### package.json（Tauri GUI側）
```json
{
  "name": "gui",
  "version": "0.1.0",
  "scripts": {
    "tauri": "tauri",
    "dev": "vite",
    "build": "tsc && vite build",
    "tauri-dev": "tauri dev",
    "tauri-build": "tauri build"
  },
  "dependencies": {
    "@tauri-apps/api": "^1.0.0",
    "react": "^18.0.0",
    "react-dom": "^18.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^1.0.0",
    "@types/react": "^18.0.0",
    "@types/react-dom": "^18.0.0",
    "typescript": "^5.0.0",
    "vite": "^4.0.0"
  }
}
```

## 🤝 コントリビューション

### 開発に参加する方法

1. **リポジトリをフォーク**
   ```bash
   git clone https://github.com/yourusername/esp32-tauri-serial.git
   cd esp32-tauri-serial
   ```

2. **フィーチャーブランチを作成**
   ```bash
   git checkout -b feature/amazing-feature
   ```

3. **変更を実装**
   - コードの品質を保つ
   - テストを追加
   - ドキュメントを更新

4. **変更をコミット**
   ```bash
   git add .
   git commit -m "Add amazing feature"
   ```

5. **プルリクエストを作成**
   ```bash
   git push origin feature/amazing-feature
   ```

### コーディング規約

- **Rust**: `cargo fmt` でフォーマット
- **TypeScript**: Prettier + ESLint
- **コミットメッセージ**: [Conventional Commits](https://www.conventionalcommits.org/)

## 📄 ライセンス

MIT License - 詳細は[LICENSE](LICENSE)ファイルを参照

## 🆘 サポート

問題や質問がある場合：

1. **既存のIssueを確認**: [Issues](../../issues)
2. **新しいIssueを作成** - 以下の情報を含める：
   - OS（macOS/Linux/Windows）
   - Rustバージョン（`rustc --version`）
   - Node.jsバージョン（`node --version`）
   - ESP32モデル
   - エラーメッセージの全文
   - 再現手順

3. **フォーラムやコミュニティ**：
   - [ESP32 Rust Community](https://matrix.to/#/#esp-rs:matrix.org)
   - [Tauri Discord](https://discord.com/invite/SpmNs4S)

---

**Happy coding with ESP32 + Tauri! 🚀**

*このライブラリがあなたのIoTプロジェクトの役に立つことを願っています。*