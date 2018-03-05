extern crate base64;

extern crate reqwest;
extern crate serde_json;
use self::reqwest::header::{Headers, Authorization, ContentType};

pub fn calc_outh2_header(consumer_key: &str, consumer_secret: &str) -> String {
    base64::encode(&format!("{}:{}", consumer_key, consumer_secret))
}

pub fn make_oauth2_header(consumer_key: &str, consumer_secret: &str) -> String {
    format!("Basic {}", calc_outh2_header(consumer_key, consumer_secret))
}

pub fn make_bearer_header(bearer_token: &str) -> String {
    format!("Bearer {}", bearer_token)
}

pub fn get_bearer_token(
    consumer_key: &str, consumer_secret: &str, uri: &str, options: &Vec<(&str, &str)>
) -> Result<String, reqwest::Error> {
    let mut headers = Headers::new();
    headers.set(Authorization(make_oauth2_header(consumer_key, consumer_secret)));
    headers.set(ContentType::form_url_encoded());
    let client = reqwest::Client::new();
    let res: serde_json::Value = client
        .post(uri)
        .query(options)
        .headers(headers)
        .send()?
        .json()?;
    Ok(String::from(res["access_token"].as_str().unwrap()))
}

pub fn get(
    bearer_token: &str, uri: &str, options: &Vec<(&str, &str)>
) -> Result<reqwest::Response, reqwest::Error> {
    let mut headers = Headers::new();
    headers.set(Authorization(format!("Bearer {}", bearer_token)));
    let client = reqwest::Client::new();
    client
        .get(uri)
        .query(options)
        .headers(headers)
        .send()
}
