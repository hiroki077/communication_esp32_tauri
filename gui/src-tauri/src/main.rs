#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{sync::{Arc, OnceLock, Mutex}, time::Duration, thread, io::{BufRead, BufReader, Write}};
use tauri::{Emitter, State};
use serde::{Deserialize, Serialize};

// 共通暗号化ライブラリ
use esp32_tauri_crypto::{CryptoSystem, EncryptedMessage, Command, Response, create_default_crypto};

// シリアルポート管理用
type SharedSerialPort = Arc<Mutex<Option<Box<dyn serialport::SerialPort>>>>;

// シリアルポート関連の型
#[derive(Debug)]
struct MessageState(String);
#[derive(Debug)]
struct PortNameState(String);

// 軽量暗号化関連の状態
struct SimpleCryptoState {
    crypto_system: CryptoSystem,
    is_ready: bool,
}

static START: OnceLock<()> = OnceLock::new();


#[tauri::command]
fn list_serial_ports() -> Result<Vec<String>, String> {
    match serialport::available_ports() {
        Ok(ports) => {
            let port_names: Vec<String> = ports.into_iter()
                .map(|p| p.port_name)
                .collect();
            Ok(port_names)
        }
        Err(e) => Err(format!("Failed to list serial ports: {}", e))
    }
}

#[tauri::command]
fn start_serial_listener(
    app: tauri::AppHandle, 
    msg_state: State<'_, Arc<Mutex<MessageState>>>, 
    port_name_state: State<'_, Arc<Mutex<PortNameState>>>,
    serial_port_state: State<'_, SharedSerialPort>,
    port_name: String
) -> Result<(), String> {
    // 二重起動を防ぐ
    if START.set(()).is_err() {
        return Ok(());
    }
    
    let shared_msg_state = msg_state.inner().clone();
    let shared_port_name_state = port_name_state.inner().clone();
    let shared_serial_port = serial_port_state.inner().clone();

    // ポート名を保存
    {
        let mut port_lock = shared_port_name_state.lock().unwrap();
        port_lock.0 = port_name.clone();
    }

    thread::spawn(move || {
        let mut reconnect_delay = 1;
        
        loop {
            match serialport::new(&port_name, 115_200)
                .timeout(Duration::from_millis(50)) // 短いタイムアウト
                .data_bits(serialport::DataBits::Eight)
                .parity(serialport::Parity::None)
                .stop_bits(serialport::StopBits::One)
                .flow_control(serialport::FlowControl::None)
                .open()
            {
                Ok(mut port) => {
                    println!("✅ Successfully opened serial port: {}", &port_name);
                    reconnect_delay = 1; // リセット
                    
                    // DTRとRTSを適切に設定
                    if let Err(e) = port.write_data_terminal_ready(true) {
                        println!("⚠️ Failed to set DTR: {}", e);
                    }
                    if let Err(e) = port.write_request_to_send(false) {
                        println!("⚠️ Failed to set RTS: {}", e);
                    }
                    
                    // ESP32の起動を待つ
                    std::thread::sleep(Duration::from_millis(100));
                    
                    // ポートを共有状態に保存（送信用）
                    let port_for_writing = port.try_clone().unwrap();
                    {
                        let mut serial_lock = shared_serial_port.lock().unwrap();
                        *serial_lock = Some(port_for_writing);
                    }
                    
                    // 受信専用でポートを使用（バイト単位で読み取り）
                    let mut buffer = [0u8; 1024];
                    let mut line_buffer = String::new();
                    
                    loop {
                        match port.read(&mut buffer) {
                            Ok(0) => {
                                // EOF時も接続は維持、少し待機
                                std::thread::sleep(Duration::from_millis(10));
                                continue;
                            }
                            Ok(bytes_read) => {
                                // 受信データを文字列として処理
                                if let Ok(received_str) = std::str::from_utf8(&buffer[..bytes_read]) {
                                    line_buffer.push_str(received_str);
                                    
                                    // 改行を見つけたら行を処理
                                    while let Some(newline_pos) = line_buffer.find('\n') {
                                        let line = line_buffer[..newline_pos].trim();
                                        if !line.is_empty() {
                                            println!("📨 Received: {}", line);
                                            
                                            // まず平文JSONレスポンスをチェック
                                            if let Ok(response) = serde_json::from_str::<Response>(line) {
                                                // 平文JSONレスポンス
                                                println!("📨 Plain JSON response received: status={}, message={}", response.status, response.message);
                                                app.emit("response-received", &response).ok();
                                                if let Ok(mut lock) = shared_msg_state.lock() {
                                                    lock.0 = format!("✅ {}", response.message);
                                                }
                                            } else if let Ok(encrypted) = serde_json::from_str::<EncryptedMessage>(line) {
                                                // 暗号化メッセージの場合、即座に復号化を試行
                                                println!("🔐 Encrypted message received, attempting decryption...");
                                                app.emit("encrypted-message-received", &encrypted).ok();
                                                
                                                // 復号化を試行
                                                match decrypt_received_message_internal(&encrypted) {
                                                    Ok(decrypted_text) => {
                                                        println!("✅ Decrypted: {}", decrypted_text);
                                                        
                                                        // 復号化されたテキストがJSONかチェック
                                                        if let Ok(response) = serde_json::from_str::<Response>(&decrypted_text) {
                                                            // JSONレスポンスの場合はメッセージ部分を表示
                                                            app.emit("response-received", &response).ok();
                                                            if let Ok(mut lock) = shared_msg_state.lock() {
                                                                lock.0 = format!("🔓 {}", response.message);
                                                            }
                                                        } else {
                                                            // 通常のテキストの場合
                                                            if let Ok(mut lock) = shared_msg_state.lock() {
                                                                lock.0 = format!("🔓 {}", decrypted_text);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        println!("❌ Decryption failed: {}", e);
                                                        if let Ok(mut lock) = shared_msg_state.lock() {
                                                            lock.0 = format!("❌ Decryption error: {}", e);
                                                        }
                                                    }
                                                }
                                            } else {
                                                // その他のメッセージ
                                                println!("📨 Raw message received: {}", line);
                                                app.emit("raw-message", line).ok();
                                                if let Ok(mut lock) = shared_msg_state.lock() {
                                                    lock.0 = line.to_string();
                                                }
                                            }
                                        }
                                        
                                        // 処理済みの行をバッファから削除
                                        line_buffer.drain(..=newline_pos);
                                    }
                                }
                            }
                            Err(e) => {
                                // タイムアウトエラーは正常動作として扱う
                                if e.kind() == std::io::ErrorKind::TimedOut {
                                    // タイムアウトは正常、接続を維持
                                    continue;
                                } else {
                                    println!("📡 Read error (non-timeout): {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    
                    // ポートをクリア
                    {
                        let mut serial_lock = shared_serial_port.lock().unwrap();
                        *serial_lock = None;
                    }
                    
                    println!("🔌 Serial connection lost, reconnecting in {}s...", reconnect_delay);
                }
                Err(e) => {
                    println!("❌ Serial open failed: {} (retry in {}s)", e, reconnect_delay);
                }
            }
            
            thread::sleep(Duration::from_secs(reconnect_delay));
            reconnect_delay = std::cmp::min(reconnect_delay + 1, 5); // 最大5秒
        }
    });

    Ok(())
}

// ESP32にコマンドを送信する関数
#[tauri::command]
fn send_command(
    serial_port_state: State<'_, SharedSerialPort>,
    action: String, 
    data: Option<String>
) -> Result<String, String> {
    let command = Command { action: action.clone(), data };
    let json_command = serde_json::to_string(&command)
        .map_err(|e| format!("JSON serialization error: {}", e))?;
    
    let mut serial_lock = serial_port_state.lock().unwrap();
    if let Some(port) = serial_lock.as_mut() {
        let command_with_newline = json_command.clone() + "\n";
        match port.write_all(command_with_newline.as_bytes()) {
            Ok(_) => {
                // フラッシュして即座に送信
                if let Err(e) = port.flush() {
                    println!("⚠️ Flush warning: {}", e);
                }
                println!("📤 Sent command: {}", json_command);
                
                // ESP32の処理時間を確保するため少し待機
                std::thread::sleep(std::time::Duration::from_millis(50));
                
                Ok(format!("Command '{}' sent successfully", action))
            }
            Err(e) => {
                println!("❌ Write error: {}", e);
                Err(format!("Failed to send command: {}", e))
            }
        }
    } else {
        Err("Serial port not connected. Please start serial listener first.".to_string())
    }
}

#[tauri::command]
fn get_message(state: State<'_, Arc<Mutex<MessageState>>>) -> Option<String> {
    state.lock().ok().map(|m| m.0.clone())
}

// 軽量暗号化システム初期化
#[tauri::command]
fn initialize_lightweight_crypto(
    crypto_state: State<'_, Arc<Mutex<SimpleCryptoState>>>
) -> Result<String, String> {
    {
        let mut crypto = crypto_state.lock().unwrap();
        crypto.crypto_system = create_default_crypto();
        crypto.is_ready = true;
    }
    
    println!("🔐 Lightweight crypto system initialized");
    Ok("Lightweight crypto system ready".to_string())
}

// 双方向通信テスト用コマンド
#[tauri::command]
fn test_bidirectional_communication(
    serial_port_state: State<'_, SharedSerialPort>
) -> Result<String, String> {
    send_command(serial_port_state, "test_bidirectional".to_string(), Some("GUI bidirectional test".to_string()))
}

// 内部復号化関数（static crypto使用）
fn decrypt_received_message_internal(encrypted: &EncryptedMessage) -> Result<String, String> {
    let crypto_system = create_default_crypto();
    crypto_system.decrypt(encrypted)
        .map_err(|e| e.to_string())
}

// 受信した暗号化メッセージを復号化
#[tauri::command]
fn decrypt_received_message(
    crypto_state: State<'_, Arc<Mutex<SimpleCryptoState>>>,
    encrypted: EncryptedMessage
) -> Result<String, String> {
    let crypto_system = {
        let crypto = crypto_state.lock().unwrap();
        if !crypto.is_ready {
            return Err("Crypto not initialized. Please initialize first.".to_string());
        }
        crypto.crypto_system.clone()
    };
    
    crypto_system.decrypt(&encrypted)
        .map_err(|e| e.to_string())
}

// 軽量暗号化コマンド送信
#[tauri::command]
fn send_lightweight_encrypted_command(
    serial_port_state: State<'_, SharedSerialPort>,
    crypto_state: State<'_, Arc<Mutex<SimpleCryptoState>>>,
    action: String,
    data: Option<String>
) -> Result<String, String> {
    // 暗号化システムを取得
    let crypto_system = {
        let crypto = crypto_state.lock().unwrap();
        if !crypto.is_ready {
            return Err("Lightweight crypto not initialized. Please initialize first.".to_string());
        }
        crypto.crypto_system.clone()
    };
    
    // コマンドを作成
    let command = Command { action: action.clone(), data };
    
    // 暗号化
    let encrypted = crypto_system.encrypt_command(&command)
        .map_err(|e| e.to_string())?;
    let encrypted_json = serde_json::to_string(&encrypted)
        .map_err(|e| format!("Encrypted message serialization error: {}", e))?;
    
    let mut serial_lock = serial_port_state.lock().unwrap();
    if let Some(port) = serial_lock.as_mut() {
        let message_with_newline = encrypted_json + "\n";
        match port.write_all(message_with_newline.as_bytes()) {
            Ok(_) => {
                if let Err(e) = port.flush() {
                    println!("⚠️ Flush warning: {}", e);
                }
                println!("🔐 Sent lightweight encrypted command: {}", action);
                
                // ESP32の処理時間を確保するため少し待機
                std::thread::sleep(std::time::Duration::from_millis(50));
                
                Ok(format!("Lightweight encrypted command '{}' sent successfully", action))
            }
            Err(e) => {
                Err(format!("Failed to send encrypted command: {}", e))
            }
        }
    } else {
        Err("Serial port not connected. Please start serial listener first.".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(MessageState(String::new()))))
        .manage(Arc::new(Mutex::new(PortNameState(String::new()))))
        .manage(Arc::new(Mutex::new(SimpleCryptoState {
            crypto_system: create_default_crypto(),
            is_ready: true,
        })))
        .manage(Arc::new(Mutex::<Option<Box<dyn serialport::SerialPort>>>::new(None)) as SharedSerialPort)
        .invoke_handler(tauri::generate_handler![
            list_serial_ports,
            start_serial_listener,
            send_command,
            get_message,
            initialize_lightweight_crypto,
            decrypt_received_message,
            send_lightweight_encrypted_command,
            test_bidirectional_communication
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri");
}