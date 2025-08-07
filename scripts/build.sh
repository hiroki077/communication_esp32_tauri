#!/bin/bash

# ESP32-Tauri暗号化通信システム ビルドスクリプト

set -e  # エラー時に終了

echo "🚀 ESP32-Tauri暗号化通信システム ビルド開始"

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

# 引数解析
BUILD_TYPE="debug"
SKIP_ESP32=false
SKIP_TAURI=false
ESP32_PORT="/dev/cu.usbserial-11230"

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --skip-esp32)
            SKIP_ESP32=true
            shift
            ;;
        --skip-tauri)
            SKIP_TAURI=true
            shift
            ;;
        --port)
            ESP32_PORT="$2"
            shift 2
            ;;
        --help)
            echo "使用方法: $0 [オプション]"
            echo ""
            echo "オプション:"
            echo "  --release      リリースビルド（デフォルト: debug）"
            echo "  --skip-esp32   ESP32ビルドをスキップ"
            echo "  --skip-tauri   Tauriビルドをスキップ"  
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

print_status "ビルド設定: $BUILD_TYPE"
print_status "ESP32ポート: $ESP32_PORT"

# プロジェクトルート確認
if [[ ! -f "README.md" ]]; then
    print_error "プロジェクトルートから実行してください"
    exit 1
fi

# 共通ライブラリテスト
print_status "共通暗号化ライブラリをテスト中..."
cd shared_crypto
cargo test
if [[ $? -eq 0 ]]; then
    print_success "共通ライブラリテスト完了"
else
    print_error "共通ライブラリテストに失敗"
    exit 1
fi
cd ..

# ESP32ビルド
if [[ "$SKIP_ESP32" == false ]]; then
    print_status "ESP32プロジェクトをビルド中..."
    cd backend
    
    if [[ "$BUILD_TYPE" == "release" ]]; then
        cargo build --release
        ESP32_BINARY="target/xtensa-esp32s3-espidf/release/backend"
    else
        cargo build
        ESP32_BINARY="target/xtensa-esp32s3-espidf/debug/backend"
    fi
    
    if [[ $? -eq 0 ]]; then
        print_success "ESP32ビルド完了"
        
        # ESP32フラッシュ（オプション）
        read -p "ESP32にフラッシュしますか？ (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_status "ESP32にフラッシュ中..."
            if espflash flash --port "$ESP32_PORT" "$ESP32_BINARY"; then
                print_success "ESP32フラッシュ完了"
            else
                print_error "ESP32フラッシュに失敗"
                exit 1
            fi
        fi
    else
        print_error "ESP32ビルドに失敗"
        exit 1
    fi
    cd ..
fi

# Tauriビルド
if [[ "$SKIP_TAURI" == false ]]; then
    print_status "Tauriプロジェクトをビルド中..."
    cd gui
    
    # 依存関係インストール
    print_status "Node.js依存関係をインストール中..."
    npm install
    
    # Tauriテスト
    print_status "Tauriバックエンドをテスト中..."
    cd src-tauri
    cargo test
    if [[ $? -ne 0 ]]; then
        print_error "Tauriテストに失敗"
        exit 1
    fi
    cd ..
    
    # ビルド実行
    if [[ "$BUILD_TYPE" == "release" ]]; then
        npm run tauri build
        if [[ $? -eq 0 ]]; then
            print_success "Tauriリリースビルド完了"
            print_status "ビルド成果物: gui/src-tauri/target/release/bundle/"
        else
            print_error "Tauriビルドに失敗"
            exit 1
        fi
    else
        print_status "開発モード用ビルドチェック..."
        npm run tauri build -- --debug
        if [[ $? -eq 0 ]]; then
            print_success "Tauriデバッグビルド完了"
        else
            print_error "Tauriビルドに失敗"
            exit 1
        fi
    fi
    cd ..
fi

# ビルド結果サマリー
echo ""
print_success "🎉 全てのビルドが完了しました！"
echo ""
echo "📦 ビルド成果物:"
if [[ "$SKIP_ESP32" == false ]]; then
    echo "   - ESP32: backend/target/xtensa-esp32s3-espidf/$BUILD_TYPE/backend"
fi
if [[ "$SKIP_TAURI" == false ]]; then
    echo "   - Tauri: gui/src-tauri/target/$BUILD_TYPE/bundle/"
fi
echo ""
print_status "使用方法:"
echo "   1. ESP32をUSBで接続"
echo "   2. シリアルポート ($ESP32_PORT) を確認"  
echo "   3. Tauriアプリを起動してテスト"
echo ""
print_status "開発モードで開始:"
echo "   cd gui && npm run tauri dev"