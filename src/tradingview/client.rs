use anyhow::Result;
use tungstenite::{connect, Message};

pub async fn lolreq() -> Result<String> {
    let body = reqwest::get("https://www.rust-lang.org")
        .await?
        .text()
        .await?;

    println!("body = {:?}", body);
    return Ok(body);
}

pub fn lol() -> Result<i32> {
    let url = "wss://data.tradingview.com/socket.io/websocket";
    let (mut socket, response) = connect(url).expect("mw");

    socket.write_message(Message::Text(
        r#"{
    "action": "authenticate",
    "data": {
        "key_id": "API-KEY",
        "secret_key": "SECRET-KEY"
    }
}"#
        .into(),
    ))?;

    socket.write_message(Message::Text(
        r#"{
    "action": "listen",
    "data": {
        "streams": ["AM.SPY"]
    }
}"#
        .into(),
    ))?;

    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }

    Ok(10)
}
