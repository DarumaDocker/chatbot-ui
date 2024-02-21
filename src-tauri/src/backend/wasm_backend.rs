use std::{collections::HashMap, sync::Arc, thread::JoinHandle};

use wasmedge_sdk::{vm::SyncInst, wasi::WasiModule, AsInstance, Module, Store, Vm};

use crate::backend::wasm_backend::chat_ui::LoadModelState;

use super::{LoadModel, LoadModelRespone, Request};

pub struct WasmBackend {
    models_dir: String,
    request_tx: crossbeam::channel::Sender<Request>,
    _manager_task: JoinHandle<()>,
}

pub struct WasmBackendManagerTask {
    pub models_dir: String,
}

impl WasmBackendManagerTask {
    fn nn_preload(&self) {
        let models = list_models(&self.models_dir);

        let preloads = models
            .into_iter()
            .map(|(path, file_stem)| {
                wasmedge_sdk::plugin::NNPreload::new(
                    file_stem,
                    wasmedge_sdk::plugin::GraphEncoding::GGML,
                    wasmedge_sdk::plugin::ExecutionTarget::AUTO,
                    path,
                )
            })
            .collect();

        wasmedge_sdk::plugin::PluginManager::nn_preload(preloads);
    }

    fn run_loop(&self, rx: crossbeam::channel::Receiver<Request>) {
        let request_rx = Arc::new(rx);
        while let Ok(req) = request_rx.recv() {
            match req {
                Request::ListModel(_) => {}
                Request::LoadModel(params, tx) => self.run_wasm(request_rx.clone(), (params, tx)),
                Request::Chat(_chatbody, tx) => {
                    let _ = tx.send(Err(super::TokenError::BackendNotRun));
                }
            }
        }
        println!("exit run_loop");
    }

    fn run_wasm(
        &self,
        request_rx: Arc<crossbeam::channel::Receiver<Request>>,
        mut load_model_req: (LoadModel, LoadModelRespone),
    ) {
        let wasm_module = Module::from_file(None, "./bk.wasm").unwrap();

        loop {
            self.nn_preload();

            let mut instances: HashMap<String, &mut (dyn SyncInst)> = HashMap::new();

            let mut wasi = create_wasi(&load_model_req.0).unwrap();
            let mut chatui =
                chat_ui::module(chat_ui::ChatBotUi::new(request_rx.clone(), load_model_req))
                    .unwrap();

            instances.insert(wasi.name().to_string(), wasi.as_mut());
            let mut wasi_nn = wasmedge_sdk::plugin::PluginManager::load_plugin_wasi_nn().unwrap();
            instances.insert(wasi_nn.name().unwrap(), &mut wasi_nn);
            instances.insert(chatui.name().unwrap(), &mut chatui);

            let store = Store::new(None, instances).unwrap();
            let mut vm = Vm::new(store);
            vm.register_module(None, wasm_module.clone()).unwrap();

            let _ = vm.run_func(None, "_start", []);
            if let Some((LoadModelState::Reload, model, tx)) =
                chatui.get_host_data_mut().load_model_state.take()
            {
                load_model_req = (model, tx);
            } else {
                break;
            }
        }

        println!("exit run_wasm");
    }
}

fn list_models(models_dir: &str) -> Vec<(String, String)> {
    let mut r = vec![];

    if let Ok(read_dir) = std::fs::read_dir(models_dir) {
        for dir_entry in read_dir {
            if let Ok(dir_entry) = dir_entry {
                let path = dir_entry.path();
                if path.is_file() && "gguf" == path.extension().unwrap_or_default() {
                    let file_stem = path.file_stem().unwrap_or(path.as_os_str());

                    r.push((
                        format!("{}", path.display()),
                        format!("{}", file_stem.to_string_lossy()),
                    ));
                }
            }
        }
    }
    r
}

fn create_wasi(load_model: &super::LoadModel) -> wasmedge_sdk::WasmEdgeResult<WasiModule> {
    let module_alias = load_model.model.clone();

    let ctx_size = load_model
        .options
        .ctx_size
        .as_ref()
        .map(ToString::to_string);

    let n_predict = load_model
        .options
        .n_predict
        .as_ref()
        .map(ToString::to_string);

    let n_gpu_layers = load_model
        .options
        .n_gpu_layers
        .as_ref()
        .map(ToString::to_string);

    let batch_size = load_model
        .options
        .batch_size
        .as_ref()
        .map(ToString::to_string);

    let temp = load_model.options.temp.as_ref().map(ToString::to_string);

    let repeat_penalty = load_model
        .options
        .repeat_penalty
        .as_ref()
        .map(ToString::to_string);

    let reverse_prompt = load_model.options.reverse_prompt.clone();

    let prompt_template = load_model.prompt_template.clone();

    let mut args = vec!["chat_ui.wasm", "-a", module_alias.as_str()];

    macro_rules! add_args {
        ($flag:expr, $value:expr) => {
            if let Some(ref value) = $value {
                args.push($flag);
                args.push(value.as_str());
            }
        };
    }

    add_args!("-c", ctx_size);
    add_args!("-n", n_predict);
    add_args!("-g", n_gpu_layers);
    add_args!("-b", batch_size);
    add_args!("--temp", temp);
    add_args!("--repeat-penalty", repeat_penalty);
    add_args!("-r", reverse_prompt);
    add_args!("-p", prompt_template);

    WasiModule::create(Some(args), None, None)
}

impl WasmBackend {
    pub fn new(models_dir: String) -> Self {
        wasmedge_sdk::plugin::PluginManager::load(None).unwrap();

        let models_dir_ = models_dir.clone();

        let (request_tx, rx) = crossbeam::channel::unbounded();
        let _manager_task = std::thread::spawn(move || {
            WasmBackendManagerTask {
                models_dir: models_dir_,
            }
            .run_loop(rx)
        });

        Self {
            models_dir,
            request_tx,
            _manager_task,
        }
    }
}

impl super::Backend for WasmBackend {
    fn request(&self, req: Request) {
        match req {
            Request::ListModel(tx) => {
                let models = list_models(&self.models_dir)
                    .into_iter()
                    .map(|(_, name)| name)
                    .collect();

                let _ = tx.send(models);
            }
            req => {
                self.request_tx.send(req).unwrap();
            }
        }
    }
}

mod chat_ui {

    use std::{io::Read, sync::Arc};

    use wasmedge_sdk::{
        error::{CoreError, CoreExecutionError},
        CallingFrame, ImportObject, Instance, WasmValue,
    };

    use crate::backend::{LoadModel, LoadModelRespone, Request, Token, TokenError};

    #[derive(Debug)]
    pub enum LoadModelState {
        Init,
        Reload,
    }

    #[derive(Debug)]
    pub struct ChatBotUi {
        pub current_req: std::io::Cursor<Vec<u8>>,
        pub request_rx: Arc<crossbeam::channel::Receiver<Request>>,
        pub token_tx: Option<crossbeam::channel::Sender<Result<Token, TokenError>>>,
        pub load_model_state: Option<(LoadModelState, LoadModel, LoadModelRespone)>,
    }

    impl ChatBotUi {
        pub fn new(
            request_rx: Arc<crossbeam::channel::Receiver<Request>>,
            load_module_req: (LoadModel, LoadModelRespone),
        ) -> Self {
            Self {
                request_rx,
                token_tx: None,
                current_req: std::io::Cursor::new(vec![]),
                load_model_state: Some((
                    LoadModelState::Init,
                    load_module_req.0,
                    load_module_req.1,
                )),
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
                // Responds to the load model request
                if let Some((LoadModelState::Init, _, ref tx)) = data.load_model_state {
                    let _ = tx.send(());
                }
                if let Ok(req) = data.request_rx.recv() {
                    match req {
                        Request::ListModel(tx) => {}
                        Request::LoadModel(params, tx) => {
                            // Exit wasm and Reload module
                            println!("wasm recv LoadModel");

                            let _ =
                                data.load_model_state
                                    .insert((LoadModelState::Reload, params, tx));

                            return Err(CoreError::Common(
                                wasmedge_sdk::error::CoreCommonError::Interrupted,
                            ));
                        }
                        Request::Chat(chatbody, tx) => {
                            // Init current_req
                            *data.current_req.get_mut() = serde_json::to_vec(&chatbody).unwrap();
                            data.current_req.set_position(0);
                            let _ = data.token_tx.insert(tx);
                        }
                    }
                }
            }

            let n = data.current_req.read(buf).unwrap();
            if n == 0 {
                data.current_req.get_mut().clear();
                data.current_req.set_position(0);
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

            let r = if buf_ptr != 0 {
                let buf = mem
                    .mut_slice::<u8>(buf_ptr, buf_size)
                    .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

                let token = unsafe { String::from_utf8_unchecked(buf.to_vec()) };
                if let Some(tx) = &data.token_tx {
                    tx.send(Ok(Token { content: token })).is_ok()
                } else {
                    false
                }
            } else {
                if let Some(tx) = &data.token_tx {
                    tx.send(Err(TokenError::EndOfSequence)).is_ok()
                } else {
                    false
                }
            };

            Ok(vec![WasmValue::from_i32(if r { 0 } else { -1 })])
        } else {
            Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
        }
    }

    fn return_token_error(
        data: &mut ChatBotUi,
        _inst: &mut Instance,
        _frame: &mut CallingFrame,
        args: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, CoreError> {
        if let Some(error_code) = args.get(0) {
            let error_code = error_code.to_i32();
            let token_err = match error_code {
                1 => TokenError::EndOfSequence,
                2 => TokenError::ContextFull,
                3 => TokenError::PromptTooLong,
                4 => TokenError::TooLarge,
                5 => TokenError::InvalidEncoding,
                _ => TokenError::Other,
            };

            if let Some(tx) = &data.token_tx {
                let _ = tx.send(Err(token_err));
            };

            Ok(vec![])
        } else {
            Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
        }
    }

    pub fn module(data: ChatBotUi) -> wasmedge_sdk::WasmEdgeResult<ImportObject<ChatBotUi>> {
        let mut module_builder = wasmedge_sdk::ImportObjectBuilder::new("chat_ui", data)?;
        module_builder.with_func::<(i32, i32), i32>("get_input", get_input)?;
        module_builder.with_func::<(i32, i32), i32>("push_token", push_token)?;
        module_builder.with_func::<i32, ()>("return_token_error", return_token_error)?;

        Ok(module_builder.build())
    }
}
