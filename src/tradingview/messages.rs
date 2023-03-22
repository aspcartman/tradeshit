use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::Deref;
use tokio_tungstenite::tungstenite::Message as WSMessage;

pub trait ToWSMessage {
    fn to_ws_message(&self) -> Result<WSMessage>;
}

impl<T> ToWSMessage for T
where
    T: ToString,
{
    fn to_ws_message(&self) -> Result<WSMessage> {
        let mut msg = self.to_string();
        if !msg.starts_with("~m~") {
            msg = format!("~m~{}~m~{}", msg.len(), msg);
        }
        Ok(WSMessage::Text(msg))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    #[serde(rename = "m")]
    pub method: String,
    #[serde(rename = "p")]
    pub parameters: Vec<Value>,
}

impl Command {
    pub fn new<S: AsRef<str>>(cmd: S, params: &[S]) -> Command {
        let params: Vec<String> = params.iter().map(|v| v.as_ref().to_owned()).collect();
        Command {
            method: cmd.as_ref().to_owned(),
            parameters: params.iter().map(|s| Value::from(s.deref())).collect(),
        }
    }
}

impl ToWSMessage for Command {
    fn to_ws_message(&self) -> Result<WSMessage> {
        serde_json::to_string(self)?.to_ws_message()
    }
}
