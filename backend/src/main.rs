use esp_idf_svc::sys::link_patches;
use std::thread;
use backend::run_communication_loop;

fn main() {
    link_patches();

    // ライブラリを使用した暗号化通信ループ
    thread::Builder::new()
        .name("esp32_crypto_communication".into())
        .stack_size(16 * 1024)
        .spawn(|| run_communication_loop(500))
        .unwrap()
        .join()
        .unwrap();
}

