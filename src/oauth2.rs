//! Application Only Authentication OAuth2
use super::TwapiError;

pub async fn get_bearer_token(
    consumer_key: &str,
    consumer_secret: &str,
) -> Result<String, TwapiError> {
    match twapi_reqwest::oauth::get_bearer_token(consumer_key, consumer_secret).await? {
        Some(access_token) => Ok(access_token),
        None => Err(TwapiError::NotExists),
    }
}
