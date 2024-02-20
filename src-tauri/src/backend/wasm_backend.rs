use std::{collections::HashMap, sync::mpsc, thread::JoinHandle};

use wasmedge_sdk::{
    vm::SyncInst, wasi::WasiModule, AsInstance, ImportObject, Module, Store, Vm, WasmEdgeResult,
    WasmValue,
};

pub struct WasmBackend {
    request_tx: mpsc::Sender<super::ChatBody>,
    token_rx: std::sync::Mutex<mpsc::Receiver<Option<String>>>,
    _wasm_task: JoinHandle<WasmEdgeResult<Vec<WasmValue>>>,
}

impl WasmBackend {
    pub fn new(wasm_filepath: &str) -> Self {
        let module = Module::from_file(None, wasm_filepath).unwrap();

        let (request_tx, request_rx) = mpsc::channel();
        let (token_tx, token_rx) = mpsc::channel();

        let _wasm_task = std::thread::spawn(move || {
            let chatbot_ui = chat_ui::ChatBotUi::new(request_rx, token_tx);
            let chatbot_module = chat_ui::module(chatbot_ui).unwrap();

            Self::run(module, chatbot_module)
        });

        Self {
            request_tx,
            token_rx: std::sync::Mutex::new(token_rx),
            _wasm_task,
        }
    }

    fn run(
        module: Module,
        mut chatui: ImportObject<chat_ui::ChatBotUi>,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        let mut instances: HashMap<String, &mut (dyn SyncInst)> = HashMap::new();
        let mut wasi = WasiModule::create(
            Some(vec!["wasm", "--log-all"]),
            None,
            Some(vec!["modules:/home/csh/ai"]),
        )
        .unwrap();

        instances.insert(wasi.name().to_string(), wasi.as_mut());
        let mut wasi_nn = wasmedge_sdk::plugin::PluginManager::load_plugin_wasi_nn().unwrap();
        instances.insert(wasi_nn.name().unwrap(), &mut wasi_nn);
        instances.insert(chatui.name().unwrap(), &mut chatui);

        let store = Store::new(None, instances).unwrap();
        let mut vm = Vm::new(store);
        vm.register_module(None, module)?;
        vm.run_func(None, "_start", [])
    }
}

impl super::Backend for WasmBackend {
    fn handler(&self, chatbody: super::ChatBody, tx: mpsc::Sender<String>) {
        let r = self.request_tx.send(chatbody);
        println!("send req {:?}", r);

        if let Ok(rx) = self.token_rx.lock() {
            while let Ok(Some(token)) = rx.recv() {
                if let Err(_) = tx.send(token) {
                    println!("send token error");
                    break;
                }
            }
        }
    }
}

mod chat_ui {

    use std::{io::Read, sync::mpsc};

    use wasmedge_sdk::{
        error::{CoreError, CoreExecutionError},
        CallingFrame, ImportObject, Instance, WasmValue,
    };

    use crate::backend;

    #[derive(Debug)]
    pub struct ChatBotUi {
        current_req: std::io::Cursor<Vec<u8>>,
        request_rx: mpsc::Receiver<backend::ChatBody>,
        token_tx: mpsc::Sender<Option<String>>,
    }

    impl ChatBotUi {
        pub fn new(
            request_rx: mpsc::Receiver<backend::ChatBody>,
            token_tx: mpsc::Sender<Option<String>>,
        ) -> Self {
            Self {
                request_rx,
                token_tx,
                current_req: std::io::Cursor::new(vec![]),
            }
        }
    }

    fn get_input(
        data: &mut ChatBotUi,
        _inst: &mut Instance,
        frame: &mut CallingFrame,
        args: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, CoreError> {
        let mem = frame
            .memory_mut(0)
            .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

        if let Some([buf_ptr, buf_size]) = args.get(0..2) {
            let buf_ptr = buf_ptr.to_i32() as usize;
            let buf_size = buf_size.to_i32() as usize;

            let buf = mem
                .mut_slice::<u8>(buf_ptr, buf_size)
                .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

            if data.current_req.get_ref().is_empty() {
                if let Ok(message) = data.request_rx.recv() {
                    *data.current_req.get_mut() = serde_json::to_vec(&message).unwrap();
                    data.current_req.set_position(0);
                }
            }

            let n = data.current_req.read(buf).unwrap();
            if n == 0 {
                data.current_req.get_mut().clear();
            }

            Ok(vec![WasmValue::from_i32(n as i32)])
        } else {
            Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
        }
    }

    fn push_token(
        data: &mut ChatBotUi,
        _inst: &mut Instance,
        frame: &mut CallingFrame,
        args: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, CoreError> {
        let mem = frame
            .memory_mut(0)
            .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

        if let Some([buf_ptr, buf_size]) = args.get(0..2) {
            let buf_ptr = buf_ptr.to_i32() as usize;
            let buf_size = buf_size.to_i32() as usize;

            let r = if buf_size != 0 {
                let buf = mem
                    .mut_slice::<u8>(buf_ptr, buf_size)
                    .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

                let token = unsafe { String::from_utf8_unchecked(buf.to_vec()) };
                data.token_tx.send(Some(token))
            } else {
                data.token_tx.send(None)
            };

            Ok(vec![WasmValue::from_i32(if r.is_ok() { 0 } else { -1 })])
        } else {
            Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
        }
    }

    pub fn module(data: ChatBotUi) -> wasmedge_sdk::WasmEdgeResult<ImportObject<ChatBotUi>> {
        let mut module_builder = wasmedge_sdk::ImportObjectBuilder::new("chat_ui", data)?;
        module_builder.with_func::<(i32, i32), i32>("get_input", get_input)?;
        module_builder.with_func::<(i32, i32), i32>("push_token", push_token)?;
        Ok(module_builder.build())
    }
}
