use std::ops::Deref;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::tradingview::messages::Command;
use crate::tradingview::quote_data::QuoteData;

pub struct QuoteSession {
    pub id: String,
    symbol: String,
    data: QuoteDataDiff,
}

impl QuoteSession {
    pub fn new(sym: &str) -> QuoteSession {
        let hash: String = rand::thread_rng().sample_iter(&Alphanumeric).take(12).map(char::from).collect();
        let mut id = "qs_".to_owned();
        id.push_str(&hash);
        QuoteSession {
            id,
            symbol: sym.to_string(),
            data: {
                let mut q = QuoteDataDiff::default();
                q.symbol = sym.to_string();
                q
            },
        }
    }

    pub fn subscribe_commands(&self) -> Vec<Command> {
        let cmd = |m: &str, p: &[&str]| {
            let mut params = vec![self.id.deref()];
            for p in p {
                params.push(p)
            }
            Command::new(m, params.deref())
        };

        vec![cmd("quote_create_session", &[]), cmd("quote_set_fields", &FIELDS), cmd("quote_add_symbols", &[self.symbol.deref()])]
    }

    pub fn process_data_update(&mut self, msg: &Map<String, Value>) -> Result<QuoteDataDiff> {
        if msg["n"].as_str().context("msg has no n")? != self.symbol {
            return Err(anyhow!("Session for {:?} received msg for {:?}", self.symbol, msg["n"].as_str()));
        }
        if msg["s"].as_str().context("no status")? != "ok" {
            return Err(anyhow!("Session for {:?} received msg with status {:?}", self.symbol, msg["s"].as_str()));
        }

        let value = msg["v"].as_object().context("no value")?;
        if let Some(v) = try { value.get("volume")?.as_f64()? } {
            self.data.volume = Some(v);
        }
        if let Some(v) = try { value.get("lp")?.as_f64()? } {
            self.data.lp = Some(v);
        }
        if let Some(v) = try { value.get("ch")?.as_f64()? } {
            self.data.ch = Some(v);
        }
        if let Some(v) = try { value.get("chp")?.as_f64()? } {
            self.data.chp = Some(v);
        }
        if let Some(v) = try { value.get("lp_time")?.as_i64()? } {
            self.data.lp_time = Some(v);
        }
        Ok(self.data.clone())
    }
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct QuoteDataDiff {
    pub symbol: String,
    pub country_code: Option<String>,
    pub currency_code: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub fractional: Option<bool>,
    pub is_tradable: Option<bool>,
    pub language: Option<String>,
    pub original_name: Option<String>,
    pub price_scale: Option<i32>,
    pub pro_name: Option<String>,
    pub provider_id: Option<String>,
    pub short_name: Option<String>,
    pub timezone: Option<String>,
    pub qtype: Option<String>,
    pub update_mode: Option<String>,
    pub current_session: Option<String>,
    pub logoid: Option<String>,
    pub lp_time: Option<i64>,
    pub ch: Option<f64>,
    pub chp: Option<f64>,
    pub high_price: Option<f64>,
    pub low_price: Option<f64>,
    pub lp: Option<f64>,
    pub open_price: Option<f64>,
    pub prev_close_price: Option<f64>,
    pub volume: Option<f64>,
}

impl QuoteDataDiff {
    pub fn to_quote(&self) -> Option<QuoteData> {
        if self.lp.is_none() || self.lp_time.is_none() {
            return None;
        }
        Some(QuoteData {
            symbol: self.symbol.to_string(),
            last_price: self.lp.unwrap(),
            last_price_time: Utc.timestamp_opt(self.lp_time.unwrap(), 0).unwrap(),
            last_update: Utc::now(), // todo fix
        })
    }
}

const FIELDS: [&'static str; 48] = [
    "base-currency-logoid",
    "ch",
    "chp",
    "currency-logoid",
    "currency_code",
    "current_session",
    "description",
    "exchange",
    "format",
    "fractional",
    "is_tradable",
    "language",
    "local_description",
    "logoid",
    "lp",
    "lp_time",
    "minmov",
    "minmove2",
    "original_name",
    "pricescale",
    "pro_name",
    "short_name",
    "type",
    "update_mode",
    "volume",
    "ask",
    "bid",
    "fundamentals",
    "high_price",
    "low_price",
    "open_price",
    "prev_close_price",
    "rch",
    "rchp",
    "rtc",
    "rtc_time",
    "status",
    "industry",
    "basic_eps_net_income",
    "beta_1_year",
    "market_cap_basic",
    "earnings_per_share_basic_ttm",
    "price_earnings_ttm",
    "sector",
    "dividends_yield",
    "timezone",
    "country_code",
    "provider_id",
];
