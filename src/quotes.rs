use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::runloop::spawn;
use crate::tradingview::client::Client;
use crate::tradingview::quote_data::QuoteData;

pub struct QuotesManager {
    tv: Client,
    quotes: Arc<Mutex<HashMap<String, QuoteData>>>,
}

impl QuotesManager {
    pub(crate) async fn new() -> QuotesManager {
        let (client, mut upds) = Client::new().await.unwrap();
        let quotes = Arc::new(Mutex::new(HashMap::new()));
        let qb = quotes.clone();
        spawn(async move {
            while let Some(quote) = upds.recv().await {
                let mut map = qb.lock().unwrap();
                map.insert(quote.symbol.to_string(), quote);
            }
            eprintln!("Trading view client returned none, worker exited")
        });

        QuotesManager { tv: client, quotes }
    }

    pub async fn subscribe(&self, sym: &str) -> Result<()> {
        self.tv.subscribe(sym).await
    }

    pub fn peek_quotes(&self, peek: impl FnOnce(&HashMap<String, QuoteData>)) {
        let quotes = self.quotes.lock().unwrap();
        peek(quotes.deref())
    }
}
