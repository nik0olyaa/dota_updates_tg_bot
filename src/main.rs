mod file_part;
mod json_part;
mod message_part;

use crate::file_part::file_work;
use dotenv::dotenv;
use log::info;
use std::env;
use std::time::Duration;

/// The URL used to fetch events related to Dota 2.
const LINK: &str =
    "https://store.steampowered.com/events/ajaxgetpartnereventspageable/?clan_accountid=0&appid=570&offset=0&count=100&l=english&origin=https:%2F%2Fwww.dota2.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    env_logger::init();

    info!("Starting main function...");

    let sleep_duration_secs = env::var("SLEEP_DURATION_SECS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(5);

    let sleep_duration = Duration::from_secs(sleep_duration_secs);

    tokio::spawn(async move {
        loop {
            info!("Starting file work...");
            file_work(LINK).await;
            info!("File work completed.");

            tokio::time::sleep(sleep_duration).await;
        }
    })
    .await?;

    info!("Main function completed.");

    Ok(())
}
