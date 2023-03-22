use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct QuoteData {
    pub symbol: String,
    pub last_price: f64,
    pub last_price_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
}
