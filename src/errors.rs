#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    ParseJsonError(#[from] serde_json::Error),

    #[error("Failed to fetch URL: {0}")]
    FetchError(#[from] reqwest::Error),
}
