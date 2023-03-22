use std::collections::HashMap;

use anyhow::{Context, Error, Result};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::tradingview::messages::{Command, ToWSMessage};
use crate::tradingview::quote_data::QuoteData;
use crate::tradingview::quote_processor::QPInput::{Subscribe, Update};
use crate::tradingview::quote_processor::{QPInput, QuoteProcessor};

type Stream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Client {
    wtx: Sender<Message>,
    qtx: Sender<QPInput>,
}

impl Client {
    pub async fn new() -> Result<(Client, Receiver<QuoteData>)> {
        let mut req = "wss://data.tradingview.com/socket.io/websocket".into_client_request()?;
        req.headers_mut().append("Origin", "https://s.tradingview.com".parse()?);
        let (stream, res) = connect_async(req).await?;

        let (stx, srx) = stream.split();
        let wtx = Self::writer(stx);
        let rrx = Self::reader(srx);
        let (qtx, qrx) = QuoteProcessor::run(wtx.clone());
        Self::processor(rrx, wtx.clone(), qtx.clone());

        let mut client = Client { wtx, qtx };
        client.write(Command::new("set_auth_token", &["unauthorized_user_token"])).await?;
        Ok((client, qrx))
    }

    fn writer(mut stx: SplitSink<Stream, Message>) -> Sender<Message> {
        let (wtx, mut wrx) = channel::<Message>(32);
        let future = async move {
            while let Some(msg) = wrx.recv().await {
                println!("--> {:?}", msg);
                if let Err(e) = stx.send(msg).await {
                    eprintln!("Error sending msg")
                }
            }
            println!("writer exit")
        };
        spawn(future);
        wtx
    }

    fn reader(mut srx: SplitStream<Stream>) -> Receiver<String> {
        let (rtx, rrx) = channel::<String>(32);
        let future = async move {
            while let Some(res) = srx.next().await {
                if let Err(e) = res {
                    eprintln!("Error reading stream {:?}", e);
                    continue;
                }

                let msg = res.ok().unwrap();
                if !msg.is_text() {
                    eprintln!("I dunno how to handle non-text msg {:?}", msg);
                    continue;
                }

                let block = msg.to_text().unwrap();
                for msg in Self::split_raw_msg(block) {
                    println!("<-- {:?}", msg);
                    if let Err(e) = rtx.send(msg.to_owned()).await {
                        println!(r#"Failed processing msg "{:?}": {:?}"#, msg, e)
                    }
                }
            }
            println!("reader exited")
        };
        spawn(future);
        rrx
    }

    fn processor(mut rrx: Receiver<String>, wtx: Sender<Message>, qtx: Sender<QPInput>) {
        let future = async move {
            while let Some(msg) = rrx.recv().await {
                let res = async {
                    // Ping
                    if msg.starts_with("~h~") {
                        wtx.send(msg.to_ws_message()?).await?;
                        return Ok(());
                    }

                    // Session info
                    if msg.contains("auth_scheme_vsn") {
                        println!("Received session info {:?}", msg);
                        return Ok(());
                    }

                    // Command
                    if msg.starts_with('{') {
                        let cmd: HashMap<String, Value> = serde_json::from_str(&msg)?;
                        // Quote Session updated
                        if cmd["m"].eq("qsd") {
                            let params = cmd["p"].as_array().context("not array")?;
                            let id = params[0].as_str().context("not string")?.to_owned();
                            let val = params.get(1).context("no data")?.to_owned();
                            qtx.send(Update(id, val)).await?;
                            return Ok(());
                        }
                    }

                    eprintln!("Msg was not processed: {:?}", msg);
                    Ok::<(), Error>(())
                }
                .await;
                if let Err(e) = res {
                    eprintln!("error processing msg: {:?}", e)
                }
            }
            println!("processor exit")
        };
        spawn(future);
    }

    async fn write(&mut self, m: impl ToWSMessage) -> Result<()> {
        self.wtx.send(m.to_ws_message()?).await?;
        Ok(())
    }

    pub async fn subscribe(&self, quote: &str) -> Result<()> {
        self.qtx.send(Subscribe(quote.to_string())).await?;
        Ok(())
    }

    fn split_raw_msg(msg: &str) -> impl Iterator<Item = &str> {
        msg.split("~m~").filter(|s| !s.is_empty()).skip(1).step_by(2)
    }
}
