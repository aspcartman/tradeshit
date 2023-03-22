use std::collections::HashMap;
use std::ops::Deref;

use anyhow::{Context, Result};
use serde_json::Value;
use tokio::spawn;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::Message;

use crate::tradingview::messages::ToWSMessage;
use crate::tradingview::quote_data::QuoteData;
use crate::tradingview::quotes::QuoteSession;

#[derive(Debug)]
pub enum QPInput {
    Subscribe(String),
    Update(String, Value),
}

pub struct QuoteProcessor {
    input: Receiver<QPInput>,
    sessions: HashMap<String, QuoteSession>,
    wstx: Sender<Message>,
    output: Sender<QuoteData>,
}

impl QuoteProcessor {
    pub fn run(wstx: Sender<Message>) -> (Sender<QPInput>, Receiver<QuoteData>) {
        let (itx, irx) = channel::<QPInput>(32);
        let (otx, orx) = channel::<QuoteData>(32);
        let mut proc = QuoteProcessor {
            input: irx,
            sessions: HashMap::new(),
            wstx,
            output: otx,
        };
        spawn(async move {
            proc.run_loop().await;
        });
        (itx, orx)
    }

    async fn run_loop(&mut self) {
        while let Some(msg) = self.input.recv().await {
            if let Err(e) = self.handle_msg(msg).await {
                eprintln!("error {:?}", e)
            }
        }
    }

    async fn handle_msg(&mut self, msg: QPInput) -> Result<()> {
        match msg {
            QPInput::Subscribe(v) => self.subscribe(&v).await,
            QPInput::Update(id, v) => self.update(&id, v).await,
        }
    }

    async fn subscribe(&mut self, symbol: &str) -> Result<()> {
        let session = QuoteSession::new(symbol);
        for command in session.subscribe_commands() {
            self.wstx.send(command.to_ws_message()?).await?;
        }
        self.sessions.insert(session.id.deref().to_string(), session);
        Ok(())
    }

    async fn update(&mut self, id: &str, update: Value) -> Result<()> {
        let session = self.sessions.get_mut(id).context("no such session")?;
        let updated = session.process_data_update(update.as_object().context("update is not an object")?)?;
        let quote = updated.to_quote();
        if let Some(q) = quote {
            self.output.send(q).await?;
        }
        Ok(())
    }
}
