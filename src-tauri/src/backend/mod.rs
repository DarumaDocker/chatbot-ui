use serde::{Deserialize, Serialize};

pub mod wasm_backend;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatBody {
    pub messages: Vec<Message>,
    pub channel_id: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Options {
    pub ctx_size: Option<u64>,
    pub n_predict: Option<u64>,
    pub n_gpu_layers: Option<u64>,
    pub batch_size: Option<u64>,
    pub temp: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub reverse_prompt: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LoadModel {
    pub model: String,
    #[serde(default)]
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub options: Options,
}

#[derive(Debug)]
pub enum TokenError {
    BackendNotRun,
    EndOfSequence,
    ContextFull,
    PromptTooLong,
    TooLarge,
    InvalidEncoding,
    Other,
}

pub struct Token {
    pub content: String,
}

pub type ListModelRespone = crossbeam::channel::Sender<Vec<String>>;
pub type LoadModelRespone = crossbeam::channel::Sender<()>;
pub type ChatRespone = crossbeam::channel::Sender<Result<Token, TokenError>>;

#[derive(Debug)]
pub enum Request {
    ListModel(ListModelRespone),
    LoadModel(LoadModel, LoadModelRespone),
    Chat(ChatBody, ChatRespone),
}

pub trait Backend {
    fn request(&self, req: Request);
}
