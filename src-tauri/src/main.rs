// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use tauri::Window;

mod backend;

struct BackendRef(pub Arc<dyn backend::Backend + Send + Sync>);

#[tauri::command(async)]
async fn send_chat_body(
    window: Window,
    backend_ref: tauri::State<'_, BackendRef>,
    chat_body: backend::ChatBody,
) -> Result<(), ()> {
    let event = format!("output/{}", chat_body.conversation_name);
    let mut rx = backend_ref.0.handler(chat_body);
    while let Some(token) = rx.recv().await {
        window.emit(&event, token).unwrap();
    }
    window.emit(&event, serde_json::Value::Null).unwrap();
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(BackendRef(Arc::new(backend::EchoBackend)))
        .invoke_handler(tauri::generate_handler![send_chat_body])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
