//! User Authentication OAuth1
extern crate base64;
extern crate chrono;
extern crate crypto;
extern crate rand;
extern crate url;
extern crate reqwest;
extern crate serde_json;

use self::rand::{Rng, thread_rng};
use self::chrono::prelude::*;
use self::crypto::sha1::Sha1;
use self::crypto::hmac::Hmac;
use self::crypto::mac::Mac;
use self::reqwest::header::{Headers, Authorization};
use std::collections::HashMap;
use super::TwapiError;

fn nonce() -> String {
    thread_rng().gen_ascii_chars().take(32).collect::<String>()
}

fn timestamp() -> String {
    format!("{}", Utc::now().timestamp())
}

fn encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>()
}

fn sign(base: &str, key: &str) -> String {
    let mut hmac = Hmac::new(Sha1::new(), key.as_bytes());
    hmac.input(base.as_bytes());    
    base64::encode(hmac.result().code())
}

fn make_query(list: &Vec<(&str, String)>, separator: &str) -> String {
    let mut result = String::from("");
    for item in list {
        if "" != result {
            result.push_str(separator);
        }
        result.push_str(&format!("{}={}", item.0, item.1));
    }
    result
}

fn calc_oauth_header(
    sign_key: &str, 
    consumer_key:&str, 
    header_options: &Vec<(&str, &str)>, 
    method: &str, 
    uri: &str, 
    options: &Vec<(&str, &str)>
) -> String {
    let mut param0: Vec<(&str, String)> = vec![
        ("oauth_consumer_key", String::from(consumer_key)),
        ("oauth_nonce", nonce()),
        ("oauth_signature_method", String::from("HMAC-SHA1")),
        ("oauth_timestamp", timestamp()),
        ("oauth_version", String::from("1.0"))
    ];
    for header_option in header_options {
        param0.push((header_option.0, encode(header_option.1)));
    }
    let mut param1 = param0.clone();
    for option in options {
        param1.push((option.0, encode(option.1)));
    }
    param1.sort();
    let parameter = make_query(&param1, "&");
    let base = format!("{}&{}&{}", method, encode(uri), encode(&parameter));
    let mut param2 = param0.clone();
    param2.push(("oauth_signature", encode(&sign(&base, sign_key))));
    make_query(&param2, ", ")
}

fn execute_token(uri: &str, signed: &str) -> Result<HashMap<String, String>, TwapiError> {
    let mut headers = Headers::new();
    headers.set(Authorization(format!("OAuth {}", signed)));
    let client = reqwest::Client::new();
    let mut response = client
        .post(uri)
        .headers(headers)
        .send()?;
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
    x_auth_access_type: Option<&str>
) -> Result<(String, String, String), TwapiError> {
    let uri = "https://api.twitter.com/oauth/request_token";
    let mut header_options = vec![("oauth_callback", oauth_callback)];
    if let Some(x_auth_access_type) = x_auth_access_type {
        header_options.push(("x_auth_access_type", x_auth_access_type));
    }
    let signed = calc_oauth_header(
        &format!("{}&", consumer_secret), 
        consumer_key, 
        &header_options,
        "POST",
        uri,
        &vec![]
    );
    let hash_query = execute_token(uri, &signed)?;
    let oauth_token = hash_query.get("oauth_token").unwrap();
    Ok((oauth_token.clone(), 
        hash_query.get("oauth_token_secret").unwrap().clone(),
        format!("http://api.twitter.com/oauth/authorize?oauth_token={}", oauth_token)
        ))
}

/// OAuth access token
/// Return oauth_token, oauth_token_secret, user_id, screen_name
pub fn access_token(
    consumer_key: &str,
    consumer_secret: &str,
    oauth_token: &str, 
    oauth_token_secret: &str, 
    oauth_verifier: &str
) -> Result<(String, String, String, String), TwapiError> {
    let uri = "https://api.twitter.com/oauth/access_token";
    let signed = calc_oauth_header(
        &format!("{}&{}", consumer_secret, oauth_token_secret), 
        consumer_key, 
        &vec![
            ("oauth_token", oauth_token),
            ("oauth_verifier", oauth_verifier),
        ],
        "POST",
        uri,
        &vec![]
    );
    let hash_query = execute_token(uri, &signed)?;
    Ok((hash_query.get("oauth_token").unwrap().clone(),
        hash_query.get("oauth_token_secret").unwrap().clone(),
        hash_query.get("user_id").unwrap().clone(),
        hash_query.get("screen_name").unwrap().clone()
        ))
}

pub struct Token {
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_token_secret: String
}

impl Token {
    pub fn new(
        consumer_key: &str, 
        consumer_secret: &str, 
        access_token: &str, 
        access_token_secret: &str
    ) -> Token {
        Token {
            consumer_key: String::from(consumer_key),
            consumer_secret: String::from(consumer_secret),
            access_token: String::from(access_token),
            access_token_secret: String::from(access_token_secret),
        }
    }

    pub fn calc_oauth_header(&self, method: &str, uri: &str, options: &Vec<(&str, &str)>) -> String {
        calc_oauth_header(
            &format!("{}&{}", &self.consumer_secret, &self.access_token_secret), 
            &self.consumer_key,
            &vec![("oauth_token",  &self.access_token)],
            method,
            uri,
            options
        )
    }

    pub fn make_oauth_header(&self, method: &str, uri: &str, options: &Vec<(&str, &str)>) -> String {
        format!("OAuth {}", self.calc_oauth_header(method, uri, options))
    }

    pub fn get(
        &self, uri: &str, options: &Vec<(&str, &str)>
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.make_oauth_header("GET", uri, &options)));
        let client = reqwest::Client::new();
        client
            .get(uri)
            .headers(headers)
            .query(&options)
            .send()
    }

    pub fn post(
        &self, uri: &str, options: &Vec<(&str, &str)>
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.make_oauth_header("POST", uri, &options)));
        let client = reqwest::Client::new();
        client
            .post(uri)
            .headers(headers)
            .form(&options)
            .send()
    }

    pub fn multipart(
        &self, uri: &str, 
        form: reqwest::multipart::Form
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.make_oauth_header("POST", uri, &vec![])));
        let client = reqwest::Client::new();
        client
            .post(uri)
            .headers(headers)
            .multipart(form)
            .send()
    }

    pub fn delete(
        &self, uri: &str, options: &Vec<(&str, &str)>
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.make_oauth_header("DELETE", uri, &options)));
        let client = reqwest::Client::new();
        client
            .delete(uri)
            .headers(headers)
            .query(&options)
            .send()
    }

    pub fn json(
        &self, uri: &str, value: &serde_json::Value
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.make_oauth_header("POST", uri, &vec![])));
        let client = reqwest::Client::new();
        client
            .post(uri)
            .headers(headers)
            .json(&value)
            .send()
    }
}