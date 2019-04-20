//! Application Only Authentication OAuth2
extern crate base64;
extern crate reqwest;
extern crate serde_json;
use self::reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use super::TwapiError;

pub fn get_bearer_token(
    consumer_key: &str, 
    consumer_secret: &str,
) -> Result<String, TwapiError> {
    let key = base64::encode(&format!("{}:{}", consumer_key, consumer_secret));
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Basic {}", key).parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded;charset=UTF-8".parse().unwrap());
    let client = reqwest::Client::new();
    let res: serde_json::Value = client
        .post("https://api.twitter.com/oauth2/token")
        .query(&vec![("grant_type", "client_credentials")])
        .headers(headers)
        .send()?
        .json()?;
    Ok(String::from(res["access_token"].as_str().unwrap()))
}