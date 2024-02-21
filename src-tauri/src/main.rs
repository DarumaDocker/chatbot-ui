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
    let event = format!("output/{}", chat_body.channel_id);
    let (tx, rx) = crossbeam::channel::unbounded();
    backend_ref.0.request(backend::Request::Chat(chat_body, tx));
    while let Ok(token) = rx.recv() {
        match token {
            Ok(token) => {
                window.emit(&event, token.content).unwrap();
            }
            Err(e) => {
                eprintln!("{event} :recv a error: {e:?}");
                break;
            }
        }
    }
    window.emit(&event, serde_json::Value::Null).unwrap();
    Ok(())
}

#[tauri::command(async)]
fn list_models(backend_ref: tauri::State<'_, BackendRef>) -> Vec<String> {
    let (tx, rx) = crossbeam::channel::unbounded();
    backend_ref.0.request(backend::Request::ListModel(tx));
    rx.recv().unwrap_or_default()
}

#[tauri::command(async)]
fn load_model(
    backend_ref: tauri::State<'_, BackendRef>,
    load_model: backend::LoadModel,
) -> Result<(), ()> {
    let (tx, rx) = crossbeam::channel::unbounded();
    backend_ref
        .0
        .request(backend::Request::LoadModel(load_model, tx));
    let r = rx.recv();
    println!("load_model {r:?}");
    r.or(Err(()))
}

fn backend_impl() -> BackendRef {
    let bk = backend::wasm_backend::WasmBackend::new("/home/csh/ai".to_string());
    BackendRef(Arc::new(bk))
}

fn main() {
    let backend = tauri::async_runtime::block_on(async { backend_impl() });
    tauri::Builder::default()
        .manage(backend)
        .invoke_handler(tauri::generate_handler![
            send_chat_body,
            list_models,
            load_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
