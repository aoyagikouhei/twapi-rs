//! A simple Twitter library. This is easy for customize.
extern crate base64;
extern crate reqwest;
extern crate serde_json;

pub mod oauth1;
pub mod oauth2;

#[cfg(feature = "account-activity")]
pub mod account_activity;

use self::reqwest::header::{Headers, Authorization};
use std::io::{BufReader, Cursor, Read};

/// Response from Twitter API
#[derive(Debug)]
pub struct TwapiResponse {
    pub status_code: u16,
    pub json: Option<serde_json::Value>
}

impl TwapiResponse {
    pub fn new(response: &mut reqwest::Response) -> TwapiResponse {
        TwapiResponse{
            status_code: response.status().as_u16(),
            json: response.json().ok()
        }
    }

    pub fn is_success(&self) -> bool {
        200 <= self.status_code && self.status_code < 300
    }
} 

/// Error in twapi library
#[derive(Debug)]
pub enum TwapiError {
    Connection(reqwest::Error),
    IO(std::io::Error),
    UrlError(reqwest::UrlError),
    Token((u16, String)),
    NotExists,
}

impl From<reqwest::Error> for TwapiError {
    fn from(err: reqwest::Error) -> TwapiError {
        TwapiError::Connection(err)
    }
}

impl From<std::io::Error> for TwapiError {
    fn from(err: std::io::Error) -> TwapiError {
        TwapiError::IO(err)
    }
}

impl From<reqwest::UrlError> for TwapiError {
    fn from(err: reqwest::UrlError) -> TwapiError {
        TwapiError::UrlError(err)
    }
}

fn make_account_activity_uri(
    command_type: &str, 
    env_name: Option<&str>, 
    file_name: Option<&str>,
) -> String {
    let prefix = match env_name {
        Some(env_name) => {
            format!(
                "https://api.twitter.com/1.1/account_activity/all/{}/{}",
                env_name,
                command_type
            )
        },
        None => format!("https://api.twitter.com/1.1/account_activity/{}", command_type)
    };
    match file_name {
        Some(file_name) => format!("{}/{}.json", prefix, file_name),
        None => format!("{}.json", prefix),
    }
}

/// Access to Twitter API
pub trait Twapi {
    fn authorization_header(
        &self, 
        method: &str, 
        uri: &str, 
        options: &Vec<(&str, &str)>
    ) -> String;

    fn get(
        &self, 
        uri: &str, 
        query_options: &Vec<(&str, &str)>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.authorization_header("GET", uri, query_options)));
        let client = reqwest::Client::new();
        client
            .get(uri)
            .headers(headers)
            .query(query_options)
            .send()
    }

    fn post(
        &self, 
        uri: &str, 
        query_options: &Vec<(&str, &str)>,
        form_options: &Vec<(&str, &str)>,
    ) -> Result<reqwest::Response, reqwest::Error>{
        let mut merged_options = query_options.clone();
        for option in form_options {
            merged_options.push(*option);
        }
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.authorization_header("POST", uri, &merged_options)));
        let client = reqwest::Client::new();
        client
            .post(uri)
            .query(query_options)
            .form(form_options)
            .headers(headers)
            .send()
    }

    fn multipart(
        &self, 
        uri: &str, 
        query_options: &Vec<(&str, &str)>,
        form: reqwest::multipart::Form,
    ) -> Result<reqwest::Response, reqwest::Error>{
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.authorization_header("POST", uri, &vec![])));
        let client = reqwest::Client::new();
        client
            .post(uri)
            .query(query_options)
            .multipart(form)
            .headers(headers)
            .send()
    }

    fn put(
        &self, 
        uri: &str, 
        query_options: &Vec<(&str, &str)>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.authorization_header("PUT", uri, query_options)));
        let client = reqwest::Client::new();
        client
            .put(uri)
            .headers(headers)
            .query(query_options)
            .send()
    }

    fn delete(
        &self, 
        uri: &str, 
        query_options: &Vec<(&str, &str)>
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.authorization_header("DELETE", uri, query_options)));
        let client = reqwest::Client::new();
        client
            .delete(uri)
            .headers(headers)
            .query(query_options)
            .send()
    }

    fn json(
        &self, 
        uri: &str, 
        query_options: &Vec<(&str, &str)>,
        json: &serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error>{
        let mut headers = Headers::new();
        headers.set(Authorization(
            self.authorization_header("POST", uri, &vec![])));
        let client = reqwest::Client::new();
        client
            .post(uri)
            .query(query_options)
            .json(&json)
            .headers(headers)
            .send()
    }

    fn get_verify_credentials(
        &self,
        params: &Vec<(&str, &str)>
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            "https://api.twitter.com/1.1/account/verify_credentials.json",
            params
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_search_tweets(
        &self,
        params: &Vec<(&str, &str)>
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            "https://api.twitter.com/1.1/search/tweets.json",
            params
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn post_statuses_update(
        &self,
        params: &Vec<(&str, &str)>
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.post(
            "https://api.twitter.com/1.1/statuses/update.json",
            &vec![],
            params
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn post_direct_messages_events_new(
        &self,
        value: &serde_json::Value
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.json(
            "https://api.twitter.com/1.1/direct_messages/events/new.json",
            &vec![],
            value
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_account_activity_subscription(
        &self,
        env_name: &str
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            format!("https://api.twitter.com/1.1/account_activity/all/{}/subscriptions.json", env_name).as_str(),
            &vec![]
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_direct_messages_welcome_messages_list(
        &self
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            "https://api.twitter.com/1.1/direct_messages/welcome_messages/list.json",
            &vec![]
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_direct_messages_welcome_messages_show(
        &self,
        id: &str
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            "https://api.twitter.com/1.1/direct_messages/welcome_messages/show.json",
            &vec![("id", id)]
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn post_direct_messages_welcome_messages_new(
        &self,
        value: &serde_json::Value
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.json(
            "https://api.twitter.com/1.1/direct_messages/welcome_messages/new.json",
            &vec![],
            value
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn delete_direct_messages_welcome_messages_destroy(
        &self,
        id: &str
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.delete(
            "https://api.twitter.com/1.1/direct_messages/welcome_messages/destroy.json",
            &vec![("id", id)]
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_media_upload(
        &self,
        media_id: &str,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            "https://upload.twitter.com/1.1/media/upload.json",
            &vec![("command", "STATUS"), ("media_id", media_id)]
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn post_media_upload(
        &self,
        file: &str,
        additional_owners: Option<String>
    ) -> Result<TwapiResponse, TwapiError> {
        let form = reqwest::multipart::Form::new()
            .file("media", file)?;
        let form = if let Some(additional_owners) = additional_owners {
            form.text("additional_owners", additional_owners)
        } else {
            form
        };
        let mut res = self.multipart(
            "https://upload.twitter.com/1.1/media/upload.json",
            &vec![],
            form
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn post_media_upload_chunk(
        &self,
        file: &str,
        media_type: &str,
        media_category: &str,
        additional_owners: Option<String>
    ) -> Result<TwapiResponse, TwapiError> {
        let metadata = std::fs::metadata(file)?;
        let file_size = metadata.len();
        let form = reqwest::multipart::Form::new()
            .text("command", "INIT")
            .text("total_bytes", file_size.to_string())
            .text("media_type", String::from(media_type))
            .text("media_category", String::from(media_category));
        let form = if let Some(additional_owners) = additional_owners {
            form.text("additional_owners", additional_owners)
        } else {
            form
        };
        let media_id = {
            let mut response = self.multipart(
                "https://upload.twitter.com/1.1/media/upload.json",
                &vec![],
                form
            )?;
            let result = TwapiResponse::new(&mut response);
            if !result.is_success() {
                return Ok(result);
            }
            String::from(
                result.json.unwrap().get("media_id_string").unwrap().as_str().unwrap())
        };

        let mut segment_index = 0;
        let f = std::fs::File::open(file)?;
        let mut reader = BufReader::new(f);
        while segment_index * 5000000 < file_size {
            let read_size: usize = if (segment_index + 1) * 5000000 < file_size {
                5000000
            } else {
                (file_size - segment_index * 5000000) as usize
            };
            let mut cursor = Cursor::new(vec![0; read_size]);
            reader.read(cursor.get_mut())?;
            let form = reqwest::multipart::Form::new()
                .text("command", "APPEND")
                .text("media_id", media_id.clone())
                .text("segment_index", segment_index.to_string())
                .part("media", reqwest::multipart::Part::reader(cursor));
            //println!("{:?}", form);
            let mut response = self.multipart(
                "https://upload.twitter.com/1.1/media/upload.json",
                &vec![],
                form
            )?;
            segment_index = segment_index + 1;
            let result = TwapiResponse::new(&mut response);
            if !result.is_success() {
                return Ok(result);
            }
        }

        let form = reqwest::multipart::Form::new()
            .text("command", "FINALIZE")
            .text("media_id", media_id);

        let mut response = self.multipart(
            "https://upload.twitter.com/1.1/media/upload.json",
            &vec![],
            form
        )?;
        Ok(TwapiResponse::new(&mut response))
    }

    fn post_media_metadata_create(
        &self,
        value: &serde_json::Value
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.json(
            "https://upload.twitter.com/1.1/media/metadata/create.json",
            &vec![],
            value
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn post_account_activity_webhooks(
        &self,
        uri: &str,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.post(
            &make_account_activity_uri("webhooks", env_name, None),
            &vec![("url", uri)], 
            &vec![]
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_account_activity_webhooks(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            &make_account_activity_uri("webhooks", env_name, None),
            &vec![],
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    // Always Fails in Standard(Beta)
    fn put_account_activity_webhooks(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.put(
            &make_account_activity_uri("webhooks", env_name, None),
            &vec![],
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn delete_account_activity_webhooks(
        &self,
        webhook_id: &str,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.delete(
            &make_account_activity_uri("webhooks", env_name, Some(webhook_id)),
            &vec![],
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn post_account_activity_subscriptions(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.post(
            &make_account_activity_uri("subscriptions", env_name, None),
            &vec![], 
            &vec![]
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_account_activity_all_count(
        &self,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            "https://api.twitter.com/1.1/account_activity/all/count.json",
            &vec![],
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_account_activity_subscriptions(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            &make_account_activity_uri("subscriptions", env_name, None),
            &vec![],
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn get_account_activity_subscriptions_list(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.get(
            &make_account_activity_uri("subscriptions", env_name, Some("list")),
            &vec![],
        )?;
        Ok(TwapiResponse::new(&mut res))
    }

    fn delete_account_activity_subscriptions(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let mut res = self.delete(
            &make_account_activity_uri("subscriptions", env_name, None),
            &vec![],
        )?;
        Ok(TwapiResponse::new(&mut res))
    }
}

/// Application Only Authenticaiton by oauth2
pub struct ApplicationAuth {
    bearer_token: String
}

impl ApplicationAuth {
    pub fn new (
        bearer_token: &str,
    ) -> ApplicationAuth {
        ApplicationAuth {
            bearer_token: String::from(bearer_token)
        }
    }   
}

impl Twapi for ApplicationAuth {
    fn authorization_header(&self, _: &str, _: &str, _: &Vec<(&str, &str)>) -> String {
        format!("Bearer {}", self.bearer_token)
    }
}

/// User Authenticaiton by oauth1
pub struct UserAuth {
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_token_secret: String,
}

impl UserAuth {
    pub fn new (
        consumer_key: &str, 
        consumer_secret: &str, 
        access_token: &str, 
        access_token_secret: &str,
    ) -> UserAuth {
        UserAuth {
            consumer_key: String::from(consumer_key),
            consumer_secret: String::from(consumer_secret),
            access_token: String::from(access_token),
            access_token_secret: String::from(access_token_secret),
        }
    }
}

impl Twapi for UserAuth {
    fn authorization_header(
        &self, 
        method: &str, 
        uri: &str, 
        options: &Vec<(&str, &str)>,
    ) -> String {
        let res = oauth1::calc_oauth_header(
            &format!("{}&{}", &self.consumer_secret, &self.access_token_secret), 
            &self.consumer_key,
            &vec![("oauth_token",  &self.access_token)],
            method,
            uri,
            options
        );
        format!("OAuth {}", res)   
    }
}