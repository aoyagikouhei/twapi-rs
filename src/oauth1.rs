//! User Authentication OAuth1
use super::TwapiError;

/// OAuth requet token
/// Return oauth_token, oauth_token_secret, uri
pub async fn request_token(
    consumer_key: &str,
    consumer_secret: &str,
    oauth_callback: &str,
    x_auth_access_type: Option<&str>,
) -> Result<(String, String, String), TwapiError> {
    let map = twapi_reqwest::oauth::request_token(
        consumer_key,
        consumer_secret,
        oauth_callback,
        x_auth_access_type,
    )
    .await?;
    let oauth_token = match map.get("oauth_token") {
        Some(oauth_token) => oauth_token,
        None => {
            return Err(TwapiError::NotExists);
        }
    };
    let oauth_token_secret = match map.get("oauth_token_secret") {
        Some(oauth_token_secret) => oauth_token_secret,
        None => {
            return Err(TwapiError::NotExists);
        }
    };
    let uri = format!(
        "http://api.twitter.com/oauth/authorize?oauth_token={}",
        oauth_token
    );

    Ok((oauth_token.to_string(), oauth_token_secret.to_string(), uri))
}

/// OAuth access token
/// Return oauth_token, oauth_token_secret, user_id, screen_name
pub async fn access_token(
    consumer_key: &str,
    consumer_secret: &str,
    oauth_token: &str,
    oauth_token_secret: &str,
    oauth_verifier: &str,
) -> Result<(String, String, String, String), TwapiError> {
    let map = twapi_reqwest::oauth::access_token(
        consumer_key,
        consumer_secret,
        oauth_token,
        oauth_token_secret,
        oauth_verifier,
    )
    .await?;
    let oauth_token = match map.get("oauth_token") {
        Some(oauth_token) => oauth_token,
        None => {
            return Err(TwapiError::NotExists);
        }
    };
    let oauth_token_secret = match map.get("oauth_token_secret") {
        Some(oauth_token) => oauth_token,
        None => {
            return Err(TwapiError::NotExists);
        }
    };
    let user_id = match map.get("user_id") {
        Some(oauth_token) => oauth_token,
        None => {
            return Err(TwapiError::NotExists);
        }
    };
    let screen_name = match map.get("screen_name") {
        Some(oauth_token) => oauth_token,
        None => {
            return Err(TwapiError::NotExists);
        }
    };
    Ok((oauth_token.to_string(), oauth_token_secret.to_string(), user_id.to_string(), screen_name.to_string()))
}
