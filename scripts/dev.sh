#!/bin/bash

# ESP32-Tauri開発モードスクリプト

set -e

echo "🔧 ESP32-Tauri開発モード開始"

# 色付きメッセージ関数
print_status() {
    echo -e "\033[1;34m[INFO]\033[0m $1"
}

print_success() {
    echo -e "\033[1;32m[SUCCESS]\033[0m $1"
}

print_error() {
    echo -e "\033[1;31m[ERROR]\033[0m $1"
}

# ESP32ポートの設定
ESP32_PORT="/dev/cu.usbserial-11230"

# 引数解析
while [[ $# -gt 0 ]]; do
    case $1 in
        --port)
            ESP32_PORT="$2"
            shift 2
            ;;
        --help)
            echo "使用方法: $0 [オプション]"
            echo ""
            echo "オプション:"
            echo "  --port PORT    ESP32シリアルポート（デフォルト: /dev/cu.usbserial-11230）"
            echo "  --help         このヘルプを表示"
            exit 0
            ;;
        *)
            print_error "不明なオプション: $1"
            exit 1
            ;;
    esac
done

# プロジェクトルート確認
if [[ ! -f "README.md" ]]; then
    print_error "プロジェクトルートから実行してください"
    exit 1
fi

# ESP32ポートの存在確認
if [[ ! -e "$ESP32_PORT" ]]; then
    print_error "ESP32ポートが見つかりません: $ESP32_PORT"
    print_status "利用可能なシリアルポート:"
    ls /dev/cu.usbserial* 2>/dev/null || echo "  (なし)"
    exit 1
fi

print_status "ESP32ポート: $ESP32_PORT"

# ESP32開発用ビルドとフラッシュ
print_status "ESP32を開発モードでビルド中..."
cd backend
cargo build

if [[ $? -eq 0 ]]; then
    print_success "ESP32ビルド完了"
    
    print_status "ESP32にフラッシュ中..."
    if espflash flash --port "$ESP32_PORT" target/xtensa-esp32s3-espidf/debug/backend; then
        print_success "ESP32フラッシュ完了"
    else
        print_error "ESP32フラッシュに失敗"
        exit 1
    fi
else
    print_error "ESP32ビルドに失敗"
    exit 1
fi
cd ..

# Tauri開発モード起動
print_status "Tauriアプリを開発モードで起動中..."
cd gui

# 依存関係の確認・インストール
if [[ ! -d "node_modules" ]]; then
    print_status "Node.js依存関係をインストール中..."
    npm install
fi

print_success "🚀 開発環境が準備できました！"
print_status "Tauriアプリが起動します..."
print_status "ESP32ポート $ESP32_PORT から暗号化メッセージを受信します"

# Tauri開発サーバー起動
npm run tauri dev