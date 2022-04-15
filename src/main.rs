use anyhow::Result;
use serenity::prelude::*;
use simple_logger::SimpleLogger;
use std::env;

use crate::discord_handler::DiscordHandler;

mod discord_handler;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable info level logging for this crate only
    SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("boom_bot", log::LevelFilter::Info)
        .init()?;

    // Get config from environment
    let token = env::var("DISCORD_TOKEN")?;
    let application_id: u64 = env::var("APP_ID")?.parse()?;

    log::info!("Parsed environment credentials");

    // Build and start the client
    let mut client = Client::builder(token)
        .event_handler(DiscordHandler)
        .application_id(application_id)
        .await?;

    log::info!("Starting client");

    if let Err(why) = client.start().await {
        log::error!("Client error: {:?}", why)
    }

    anyhow::Ok(())
}
