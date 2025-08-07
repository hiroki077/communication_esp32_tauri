//! # ESP32平文通信ライブラリ
//!
//! ESP32でTauriアプリケーションとの平文双方向通信を行うためのライブラリです。

use esp_idf_svc::hal::delay::FreeRtos;
use esp32_tauri_crypto::{Command, Response};
use serde_json;
use log;
use std::io::{BufRead, BufReader, stdin};

// Command と Response は共通ライブラリから取得

/// レスポンス送信関数
fn send_response(status: &str, message: &str, response_to: Option<&str>) {
    let response = Response {
        status: status.to_string(),
        message: message.to_string(),
        response_to: response_to.map(|s| s.to_string()),
    };
    
    if let Ok(json) = serde_json::to_string(&response) {
        println!("{}", json);
    }
}

/// 受信したコマンドを処理
fn process_command(command: &Command) {
    // デバッグ情報はログのみに出力（シリアルには送信しない）
    log::info!("📨 Processing command: action='{}', data={:?}", command.action, command.data);
    
    match command.action.as_str() {
        "hello" => {
            log::info!("👋 Processing hello command");
            send_response("hello_response", "🎉 Hello from ESP32! Bidirectional crypto communication works!", Some("hello"));
        }
        "ping" => {
            log::info!("🏓 Processing ping command");
            send_response("pong", "🏓 Pong from ESP32!", Some("ping"));
        }
        "status" => {
            log::info!("📊 Processing status command");
            send_response("status_response", "✅ ESP32 is running normally", Some("status"));
        }
        _ => {
            log::warn!("❓ Unknown command: {}", command.action);
            send_response("error", "Unknown command", Some(&command.action));
        }
    }
}

/// 受信した行を処理
fn process_line(line: &str) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }
    
    log::info!("📨 Received line: '{}'", trimmed);
    
    match serde_json::from_str::<Command>(trimmed) {
        Ok(command) => {
            process_command(&command);
        }
        Err(e) => {
            log::error!("❌ Failed to parse JSON command: {}", e);
            send_response("error", "Invalid JSON format", None);
        }
    }
}

/// ESP32でのシンプルなUART通信ループ（平文）
/// 
/// 標準入力からのコマンドを受信し、標準出力に応答を送信します。
pub fn run_plain_uart_loop() -> ! {
    // 起動通知（JSONレスポンスのみ送信）
    send_response("ready", "ESP32 ready for commands", None);
    
    let stdin = stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    
    loop {
        line.clear();
        
        // 標準入力から1行読み取り
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF - 少し待機してリトライ
                FreeRtos::delay_ms(10);
                continue;
            }
            Ok(_) => {
                // 行を処理
                process_line(&line);
            }
            Err(e) => {
                // WouldBlock エラーは正常（ノンブロッキング読み取り）
                match e.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        // 正常なタイムアウト、何もしない
                    }
                    _ => {
                        // エラーはJSON形式で送信
                        send_response("error", "UART read error occurred", None);
                    }
                }
                FreeRtos::delay_ms(10);
                continue;
            }
        }
        
        // 短い遅延でWDTを避ける
        FreeRtos::delay_ms(2);
    }
}

/// 後方互換性のための関数（従来のインターフェース）
pub fn run_communication_loop(_interval_ms: u32) {
    run_plain_uart_loop();
}