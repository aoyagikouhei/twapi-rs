//! User Authentication OAuth1
use reqwest;
use url;

use twapi_oauth;

use self::reqwest::header::{HeaderMap, AUTHORIZATION};
use super::TwapiError;
use std::collections::HashMap;

fn execute_token(uri: &str, signed: &str) -> Result<HashMap<String, String>, TwapiError> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("OAuth {}", signed).parse().unwrap());
    let client = reqwest::Client::new();
    let mut response = client.post(uri).headers(headers).send()?;
    let status_code = response.status().as_u16();
    if status_code < 200 || status_code >= 300 {
        return Err(TwapiError::Token((status_code, response.text()?)));
    }
    let parsed_url = url::Url::parse(&format!("http://127.0.0.1/?{}", response.text()?))?;
    Ok(parsed_url.query_pairs().into_owned().collect())
}

/// OAuth requet token
/// Return oauth_token, oauth_token_secret, uri
pub fn request_token(
    consumer_key: &str,
    consumer_secret: &str,
    oauth_callback: &str,
    x_auth_access_type: Option<&str>,
) -> Result<(String, String, String), TwapiError> {
    let uri = "https://api.twitter.com/oauth/request_token";
    let mut header_options = vec![("oauth_callback", oauth_callback)];
    if let Some(x_auth_access_type) = x_auth_access_type {
        header_options.push(("x_auth_access_type", x_auth_access_type));
    }
    let signed = twapi_oauth::calc_oauth_header(
        &format!("{}&", consumer_secret),
        consumer_key,
        &header_options,
        "POST",
        uri,
        &vec![],
    );
    let hash_query = execute_token(uri, &signed)?;
    let oauth_token = hash_query.get("oauth_token").unwrap();
    Ok((
        oauth_token.clone(),
        hash_query.get("oauth_token_secret").unwrap().clone(),
        format!(
            "http://api.twitter.com/oauth/authorize?oauth_token={}",
            oauth_token
        ),
    ))
}

/// OAuth access token
/// Return oauth_token, oauth_token_secret, user_id, screen_name
pub fn access_token(
    consumer_key: &str,
    consumer_secret: &str,
    oauth_token: &str,
    oauth_token_secret: &str,
    oauth_verifier: &str,
) -> Result<(String, String, String, String), TwapiError> {
    let uri = "https://api.twitter.com/oauth/access_token";
    let signed = twapi_oauth::calc_oauth_header(
        &format!("{}&{}", consumer_secret, oauth_token_secret),
        consumer_key,
        &vec![
            ("oauth_token", oauth_token),
            ("oauth_verifier", oauth_verifier),
        ],
        "POST",
        uri,
        &vec![],
    );
    let hash_query = execute_token(uri, &signed)?;
    Ok((
        hash_query.get("oauth_token").unwrap().clone(),
        hash_query.get("oauth_token_secret").unwrap().clone(),
        hash_query.get("user_id").unwrap().clone(),
        hash_query.get("screen_name").unwrap().clone(),
    ))
}
