//! Application Only Authentication OAuth2
extern crate base64;
extern crate reqwest;
extern crate serde_json;
use self::reqwest::header::{Headers, Authorization, ContentType};
use super::TwapiError;

pub fn get_bearer_token(
    consumer_key: &str, 
    consumer_secret: &str,
) -> Result<String, TwapiError> {
    let key = base64::encode(&format!("{}:{}", consumer_key, consumer_secret));
    let mut headers = Headers::new();
    headers.set(Authorization(format!("Basic {}", key)));
    headers.set(ContentType::form_url_encoded());
    let client = reqwest::Client::new();
    let res: serde_json::Value = client
        .post("https://api.twitter.com/oauth2/token")
        .query(&vec![("grant_type", "client_credentials")])
        .headers(headers)
        .send()?
        .json()?;
    Ok(String::from(res["access_token"].as_str().unwrap()))
}