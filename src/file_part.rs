use crate::errors::AppError;
use crate::json_part::read_page_to_json_str_headlines;
use crate::message_part::send_first_upd;
use log::{error, info};
use serde_json::{self, Value};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

const FILE1: &str = "temp_new.json";
const FILE2: &str = "temp_old.json";

/// Writes headlines to a JSON file.
///
/// This function writes the provided headlines to a JSON file. It converts the headlines into
/// a JSON string using `serde_json::to_string()` and writes the string to the specified file.
/// Returns `Ok(())` if the operation succeeds, otherwise returns an `io::Error`.
pub async fn write_headlines_to_json_file(headlines: Vec<String>) -> Result<(), AppError> {
    info!("Writing headlines to JSON file.");
    let json_str = serde_json::to_string(&headlines).map_err(AppError::ParseJsonError)?;

    let mut file = File::create("temp_new.json")?;
    file.write_all(json_str.as_bytes())?;
    info!("Headlines successfully written to JSON file.");

    Ok(())
}

/// Reads the content of a file into a string.
///
/// This function reads the content of the specified file into a string. Returns `Ok(content)`
/// if the operation succeeds, otherwise returns an `io::Error`.
fn read_file_content(filename: &str) -> Result<String, AppError> {
    info!("Reading content from file: {}", filename);
    let mut file = File::open(filename).map_err(AppError::IoError)?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(AppError::IoError)?;
    info!("Content read from file: {}", filename);
    Ok(content)
}

/// Parses a JSON string into a `serde_json::Value`.
///
/// This function parses the provided JSON string into a `serde_json::Value`. Returns `Ok(value)`
/// if the operation succeeds, otherwise returns a `serde_json::Error`.
fn parse_json(content: &str) -> Result<Value, serde_json::Error> {
    info!("Parsing JSON.");
    serde_json::from_str(content)
}

/// Compares the content of two JSON files.
///
/// This function compares the content of two JSON files. It returns `Ok(true)` if the files are
/// equal, `Ok(false)` if they are different, and an error message if any error occurs during
/// the comparison.
fn compare_json_files(file1: &str, file2: &str) -> Result<bool, String> {
    info!("Comparing JSON files: {} and {}", file1, file2);
    let content1 = read_file_content(file1)
        .map_err(|err| format!("Failed to read file {}: {}", file1, err))?;
    let content2 = read_file_content(file2)
        .map_err(|err| format!("Failed to read file {}: {}", file2, err))?;
    let json_value1 = parse_json(&content1)
        .map_err(|err| format!("Failed to parse JSON from file {}: {}", file1, err))?;
    let json_value2 = parse_json(&content2)
        .map_err(|err| format!("Failed to parse JSON from file {}: {}", file2, err))?;
    info!("Comparison complete.");
    Ok(json_value1 == json_value2)
}

/// Performs file-related tasks.
///
/// This function performs file-related tasks including reading headlines from a web page,
/// writing them to a JSON file, comparing JSON files, removing and renaming files, and sending
/// updates via Telegram. It logs information about each step and any errors encountered.
pub async fn file_work(url: &str) {
    info!("Starting file work...");
    let headlines = read_page_to_json_str_headlines(url)
        .await
        .expect("Failed to read headlines from page");
    write_headlines_to_json_file(headlines)
        .await
        .expect("Failed to write headlines to JSON file");

    match compare_json_files(FILE1, FILE2) {
        Ok(true) => info!("The JSON files are equal. Nothing new."),
        Ok(false) => {
            info!("The JSON files are different.");

            if let Err(err) = fs::remove_file(FILE2) {
                error!("Failed to remove file {}: {}", FILE2, err);
            }
            if let Err(err) = fs::rename(FILE1, FILE2) {
                error!("Failed to rename file {}: {}", FILE1, err);
            }
            send_first_upd().await;
        }
        Err(err) => println!("Error: {}", err),
    }
    info!("File work completed.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_write_headlines_to_json_file() {
        let headlines = vec!["headline1".to_string(), "headline2".to_string()];
        assert!(write_headlines_to_json_file(headlines).await.is_ok());
    }

    #[test]
    fn test_read_file_content() {
        let expected_content =
            r#"["Dota 2 Update 3/28/2024","Gameplay Patch 7.35d And Matchmaking Features"]"#;
        let content = read_file_content("test_files/test1_eq.json").unwrap();
        assert_eq!(content, expected_content);
    }

    #[test]
    fn test_parse_json() {
        let content = "{\"key\":\"value\"}";
        let parsed_json = parse_json(content).unwrap();
        assert_eq!(parsed_json, json!({"key": "value"}));
    }

    #[test]
    fn test_compare_json_files() {
        let result =
            compare_json_files("test_files/test1_eq.json", "test_files/test2_eq.json").unwrap();
        assert!(result);

        let result =
            compare_json_files("test_files/test1_eq.json", "test_files/test_dif.json").unwrap();
        assert!(!result);
    }
}
