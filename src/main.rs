#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![feature(try_blocks)]

use anyhow::{anyhow, Result};

use app::AppState;

use crate::quotes::QuotesManager;

mod app;
mod quotes;
mod runloop;
mod tradingview;
mod view;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let opts = eframe::NativeOptions::default();
    let manager = QuotesManager::new().await;
    let syms = ["MOEX:NGH2023", "MOEX:NGJ2023", "MOEX:SVM2023", "MOEX:GDM2023", "MOEX:PDM2023", "MOEX:PTM2023"];
    for s in syms {
        manager.subscribe(s).await?;
    }
    eframe::run_native("TradeShit", opts, Box::new(|cc| Box::new(AppState::new(cc, manager)))).map_err(|e| anyhow!("{:?}", e))?;
    Ok(())
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(AppState::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
