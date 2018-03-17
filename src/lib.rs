//! A simple Twitter library. This is easy for customize.
extern crate base64;
extern crate reqwest;
extern crate serde_json;

pub mod oauth1;
pub mod oauth2;

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
    Token(String),
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

/// Access to Twitter API
pub trait Twapi {
    fn get(&self, _: &str, _: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError> {
        Err(TwapiError::NotExists)
    }
    fn post(&self, _: &str, _: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError>{
        Err(TwapiError::NotExists)
    }
    fn multipart(&self, _: &str, _: reqwest::multipart::Form) -> Result<reqwest::Response, TwapiError>{
        Err(TwapiError::NotExists)
    }
    fn delete(&self, _: &str, _: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError>{
        Err(TwapiError::NotExists)
    }
    fn json(&self, _: &str, _: &serde_json::Value) -> Result<reqwest::Response, TwapiError>{
        Err(TwapiError::NotExists)
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
            value
        )?;
        Ok(TwapiResponse::new(&mut res))
    }
}

/// User Authenticaiton by oauth1
pub struct UserAuth {
    token: self::oauth1::Token
}

impl UserAuth {
    pub fn new (
        consumer_key: &str, 
        consumer_secret: &str, 
        access_token: &str, 
        access_token_secret: &str
    ) -> UserAuth {
        UserAuth {
            token: self::oauth1::Token::new(
                consumer_key,
                consumer_secret,
                access_token,
                access_token_secret
            )
        }
    }
}

impl Twapi for UserAuth {
    fn get(&self, uri: &str, options: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError> {
        Ok(self.token.get(uri, options)?)
    }

    fn post(&self, uri: &str, options: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError> {
        Ok(self.token.post(uri, options)?)
    }

    fn multipart(&self, uri: &str, form: reqwest::multipart::Form) -> Result<reqwest::Response, TwapiError>{
        Ok(self.token.multipart(uri, form)?)
    }

    fn delete(&self, uri: &str, options: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError> {
        Ok(self.token.delete(uri, options)?)
    }

    fn json(&self, uri: &str, value: &serde_json::Value) -> Result<reqwest::Response, TwapiError> {
        Ok(self.token.json(uri, value)?)
    }
    
}

/// Application Only Authenticaiton by oauth2
pub struct ApplicationAuth {
    baerer_token: String
}

impl ApplicationAuth {
    pub fn new (
        baerer_token: &str
    ) -> ApplicationAuth {
        ApplicationAuth {
            baerer_token: String::from(baerer_token)
        }
    }

    pub fn new_with_consumer (
        consumer_key: &str, 
        consumer_secret: &str
    ) -> ApplicationAuth {
        let baerer_token = self::oauth2::get_bearer_token(
            consumer_key, 
            consumer_secret, 
            "https://api.twitter.com/oauth2/token", 
            &vec![("grant_type", "client_credentials")]);
        ApplicationAuth {
            baerer_token: baerer_token.unwrap()
        }
    }
}

impl Twapi for ApplicationAuth {
    fn get(&self, uri: &str, options: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError> {
        Ok(self::oauth2::get(&self.baerer_token, uri, options)?)
    }
}