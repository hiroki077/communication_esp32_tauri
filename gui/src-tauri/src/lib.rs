use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{State, Emitter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    text: String,
}

type MessageState = Arc<Mutex<Option<String>>>;

#[tauri::command]
fn get_message(state: State<MessageState>) -> Option<String> {
    state.lock().unwrap().clone()
}

#[tauri::command]
async fn start_serial_listener(app: tauri::AppHandle, state: State<'_, MessageState>) -> Result<(), String> {
    let state = state.inner().clone();
    
    tokio::spawn(async move {
        loop {
            match serialport::new("/dev/cu.usbserial-11210", 115_200)
                .timeout(Duration::from_millis(1000))
                .open()
            {
                Ok(mut port) => {
                    let mut buffer = Vec::new();
                    loop {
                        let mut byte = [0u8; 1];
                        match port.read(&mut byte) {
                            Ok(_) => {
                                if byte[0] == b'\n' {
                                    if let Ok(line) = String::from_utf8(buffer.clone()) {
                                        if let Ok(message) = serde_json::from_str::<Message>(&line) {
                                            *state.lock().unwrap() = Some(message.text.clone());
                                            let _ = app.emit("message-received", &message.text);
                                        }
                                    }
                                    buffer.clear();
                                } else {
                                    buffer.push(byte[0]);
                                }
                            },
                            Err(_) => break,
                        }
                    }
                },
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let message_state: MessageState = Arc::new(Mutex::new(None));
    
    tauri::Builder::default()
        .manage(message_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_message, start_serial_listener])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
