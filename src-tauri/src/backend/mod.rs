use serde::{Deserialize, Serialize};

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
    #[serde(rename = "conversationName")]
    pub conversation_name: String,
}

pub trait Backend {
    fn handler(&self, chatbody: ChatBody) -> tokio::sync::mpsc::UnboundedReceiver<String>;
}

pub struct EchoBackend;

impl Backend for EchoBackend {
    fn handler(&self, chatbody: ChatBody) -> tokio::sync::mpsc::UnboundedReceiver<String> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            let message = chatbody.messages.last();
            if let Some(message) = message {
                for s in message.content.chars() {
                    let _ = tx.send(s.to_string());
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
            }
        });
        rx
    }
}
