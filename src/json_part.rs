use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct AnnouncementBody {
    pub body: Value,
    pub headline: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub announcement_body: AnnouncementBody,
}

/// Reads a page to JSON string and extracts events.
///
/// This function fetches the specified URL, parses the JSON response, and deserializes it into
/// a vector of `Event` structs. It returns a `Result` containing either the vector of events or
/// an `std::io::Error` if the operation fails.
pub async fn read_page_to_json_str_events(url: &str) -> Result<Vec<Event>, std::io::Error> {
    info!("Fetching URL: {}", url);
    let response = reqwest::get(url).await.expect("Failed to fetch URL");
    info!("URL fetched successfully");
    info!("Parse JSON from response");
    let json: Value = response.json().await.expect("Failed to parse JSON");
    info!("Convert JSON to string");
    let json_str = serde_json::to_string(&json).expect("Failed to convert JSON to string");
    let json: Value = serde_json::from_str(&json_str)?;
    info!("Deserialize events");
    let events: Vec<Event> = serde_json::from_value(json["events"].clone())?;
    info!("Events read successfully");
    Ok(events)
}

/// Reads a page to JSON string and extracts headlines.
///
/// This function fetches the specified URL, parses the JSON response, and extracts the headlines
/// from the events. It returns a `Result` containing either the vector of headlines or
/// error if the operation fails.
pub async fn read_page_to_json_str_headlines(
    url: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    info!("Fetching URL: {}", url);
    let response = reqwest::get(url).await?;
    info!("URL fetched successfully");
    info!("Parse JSON from response");
    let json: Value = response.json().await?;
    info!("Convert JSON to string");
    let json_str = serde_json::to_string(&json)?;
    let json: Value = serde_json::from_str(&json_str)?;

    let events_json = &json["events"];
    let headlines: Vec<String> = events_json
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|event_json| {
            event_json["announcement_body"]["headline"]
                .as_str()
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    info!("Headlines read successfully");

    Ok(headlines)
}
