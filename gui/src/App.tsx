import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

// ESP32レスポンス型定義
interface ESP32Response {
  status: string;
  message: string;
  timestamp: number;
  response_to?: string;
}

// 暗号化メッセージ型定義
interface EncryptedMessage {
  ciphertext: string;
  nonce: string;
}


function App() {
  const [message, setMessage] = useState<string>("");
  const [serialPorts, setSerialPorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState<string>("");
  const [isListening, setIsListening] = useState<boolean>(false);

  useEffect(() => {
    // JSON レスポンス受信リスナー
    const responseListener = listen<ESP32Response>("response-received", (event) => {
      const response = event.payload;
      setMessage(response.message);
    });

    // 生メッセージ受信リスナー
    const rawListener = listen<string>("raw-message", (event) => {
      setMessage(event.payload);
    });

    // 暗号化メッセージ受信リスナー
    const encryptedListener = listen<EncryptedMessage>("encrypted-message-received", (event) => {
      // 復号化を試行
      decryptReceivedMessage(event.payload);
    });


    // Load available serial ports on startup
    loadSerialPorts();

    return () => {
      responseListener.then(f => f());
      rawListener.then(f => f());
      encryptedListener.then(f => f());
    };
  }, []);

  const loadSerialPorts = async () => {
    try {
      const ports = await invoke<string[]>("list_serial_ports");
      setSerialPorts(ports);
      if (ports.length > 0 && !selectedPort) {
        // Auto-select ESP32-like ports
        const esp32Port = ports.find(p => 
          p.includes("usbserial") || 
          p.includes("ttyUSB") || 
          p.includes("ttyACM")
        );
        setSelectedPort(esp32Port || ports[0]);
      }
    } catch (error) {
      console.error("Failed to load serial ports:", error);
    }
  };

  const getLatestMessage = async () => {
    try {
      const latestMessage = await invoke<string | null>("get_message");
      if (latestMessage) {
        setMessage(latestMessage);
      }
    } catch (error) {
      console.error("Failed to get message:", error);
    }
  };

  const startSerialListener = async () => {
    if (!selectedPort) {
      alert("Please select a serial port first");
      return;
    }
    
    try {
      await invoke("start_serial_listener", { portName: selectedPort });
      setIsListening(true);
      console.log(`Serial listener started on ${selectedPort}`);
    } catch (error) {
      console.error("Failed to start serial listener:", error);
      alert(`Failed to start serial listener: ${error}`);
    }
  };

  // 暗号化メッセージを復号化
  const decryptReceivedMessage = async (encryptedMsg: EncryptedMessage) => {
    try {
      const decrypted = await invoke<string>("decrypt_received_message", { 
        encrypted: encryptedMsg
      });
      setMessage(`🔓 ${decrypted}`);
    } catch (error) {
      console.error("復号化に失敗:", error);
      setMessage(`❌ 復号化エラー: ${error}`);
    }
  };

  // ESP32にコマンドを送信する関数
  const sendCommand = async (action: string, data?: string) => {
    try {
      const result = await invoke<string>("send_command", { 
        action,
        data: data || null
      });
      console.log(result);
    } catch (error) {
      console.error("Failed to send command:", error);
      alert(`コマンド送信に失敗: ${error}`);
    }
  };



  return (
    <main className="container">
      <h1>ESP32 Message Display</h1>
      
      {/* Serial Port Selection */}
      <div style={{ 
        padding: "15px", 
        border: "1px solid #ddd", 
        borderRadius: "8px", 
        backgroundColor: "#f8f9fa",
        margin: "20px 0"
      }}>
        <h3>Serial Port Configuration</h3>
        <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "10px" }}>
          <label>Port:</label>
          <select 
            value={selectedPort} 
            onChange={(e) => setSelectedPort(e.target.value)}
            style={{
              padding: "8px 12px",
              fontSize: "14px",
              border: "1px solid #ccc",
              borderRadius: "4px",
              minWidth: "200px"
            }}
          >
            {serialPorts.map(port => (
              <option key={port} value={port}>{port}</option>
            ))}
          </select>
          <button onClick={loadSerialPorts} style={{
            padding: "8px 12px",
            fontSize: "14px",
            backgroundColor: "#6c757d",
            color: "white",
            border: "none",
            borderRadius: "4px",
            cursor: "pointer"
          }}>
            Refresh
          </button>
        </div>
        <div style={{ fontSize: "14px", color: "#666" }}>
          Status: {isListening ? "🟢 接続中" : "🔴 停止中"}
        </div>
      </div>

      {/* Message Display */}
      <div style={{ 
        padding: "20px", 
        border: "2px solid #007bff", 
        borderRadius: "10px", 
        backgroundColor: "#ffffff",
        textAlign: "center",
        margin: "20px 0",
        minHeight: "100px"
      }}>
        <h2 style={{ color: "#007bff", marginBottom: "15px" }}>ESP32からのメッセージ:</h2>
        <div style={{ 
          fontSize: "20px", 
          fontWeight: "bold", 
          color: "#000000",
          minHeight: "40px",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#f8f9fa",
          border: "1px solid #dee2e6",
          borderRadius: "5px",
          padding: "10px"
        }}>
          {message || "メッセージを待っています..."}
        </div>
      </div>

      {/* Control Buttons */}
      <div style={{ display: "flex", gap: "10px", justifyContent: "center", marginBottom: "20px" }}>
        <button 
          onClick={startSerialListener} 
          disabled={isListening || !selectedPort}
          style={{
            padding: "10px 20px",
            fontSize: "16px",
            backgroundColor: isListening ? "#6c757d" : "#2196F3",
            color: "white",
            border: "none",
            borderRadius: "5px",
            cursor: isListening ? "not-allowed" : "pointer",
            opacity: isListening ? 0.6 : 1
          }}
        >
          {isListening ? "Listening..." : "Start Serial Listener"}
        </button>
        
        <button onClick={getLatestMessage} style={{
          padding: "10px 20px",
          fontSize: "16px",
          backgroundColor: "#4CAF50",
          color: "white",
          border: "none",
          borderRadius: "5px",
          cursor: "pointer"
        }}>
          Get Latest Message
        </button>
      </div>

      {/* Simple Button */}
      {isListening && (
        <div style={{ 
          textAlign: "center",
          margin: "30px 0"
        }}>
          <button 
            onClick={() => sendCommand("hello")}
            style={{ 
              padding: "20px 50px", 
              fontSize: "24px",
              backgroundColor: "#28a745", 
              color: "white", 
              border: "none", 
              borderRadius: "10px", 
              cursor: "pointer",
              fontWeight: "bold",
              boxShadow: "0 4px 8px rgba(0,0,0,0.2)"
            }}
          >
            👋 Hello
          </button>
        </div>
      )}

    </main>
  );
}

export default App;
