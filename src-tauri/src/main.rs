// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use tauri::Window;

mod backend;

struct BackendRef(pub Arc<dyn backend::Backend + Send + Sync>);

#[tauri::command(async)]
fn send_chat_body(
    window: Window,
    backend_ref: tauri::State<'_, BackendRef>,
    chat_body: backend::ChatBody,
) -> Result<(), ()> {
    let event = format!("output/{}", chat_body.conversation_name);
    let (tx, rx) = std::sync::mpsc::channel();
    let bk = backend_ref.0.clone();
    std::thread::spawn(move || {
        bk.handler(chat_body, tx);
    });
    while let Ok(token) = rx.recv() {
        window.emit(&event, token).unwrap();
    }
    window.emit(&event, serde_json::Value::Null).unwrap();
    Ok(())
}

fn backend_impl() -> BackendRef {
    wasmedge_sdk::plugin::PluginManager::load(None).unwrap();

    wasmedge_sdk::plugin::PluginManager::nn_preload(vec![wasmedge_sdk::plugin::NNPreload::new(
        "default",
        wasmedge_sdk::plugin::GraphEncoding::GGML,
        wasmedge_sdk::plugin::ExecutionTarget::AUTO,
        "/home/csh/ai/llama-2-7b-chat.Q5_K_M.gguf",
    )]);

    let bk = backend::wasm_backend::WasmBackend::new("./bk.wasm");
    BackendRef(Arc::new(bk))
}

fn main() {
    let backend = tauri::async_runtime::block_on(async { backend_impl() });
    tauri::Builder::default()
        .manage(backend)
        .invoke_handler(tauri::generate_handler![send_chat_body])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
