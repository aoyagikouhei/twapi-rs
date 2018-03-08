//! A simple Twitter library. This is easy for customize.
extern crate base64;
extern crate reqwest;
extern crate serde_json;

pub mod oauth1;
pub mod oauth2;

use std::io::{BufReader, Cursor, Read};

#[derive(Debug)]
pub enum TwapiError {
    Connection(reqwest::Error),
    JSON(serde_json::Error),
    IO(std::io::Error),
    NotExists,
}

impl From<reqwest::Error> for TwapiError {
    fn from(err: reqwest::Error) -> TwapiError {
        TwapiError::Connection(err)
    }
}

impl From<serde_json::Error> for TwapiError {
    fn from(err: serde_json::Error) -> TwapiError {
        TwapiError::JSON(err)
    }
}

impl From<std::io::Error> for TwapiError {
    fn from(err: std::io::Error) -> TwapiError {
        TwapiError::IO(err)
    }
}

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

    fn get_search_tweets(
        &self,
        params: &Vec<(&str, &str)>
    ) -> Result<serde_json::Value, TwapiError> {
        let res = self.get(
            "https://api.twitter.com/1.1/search/tweets.json",
            params
        )?.json()?;
        Ok(res)
    }

    fn post_statuses_update(
        &self,
        params: &Vec<(&str, &str)>
    ) -> Result<serde_json::Value, TwapiError> {
        let res = self.post(
            "https://api.twitter.com/1.1/statuses/update.json",
            params
        )?.json()?;
        Ok(res)
    }

    fn post_direct_messages_events_new(
        &self,
        value: &serde_json::Value
    ) -> Result<serde_json::Value, TwapiError> {
        let res = self.json(
            "https://api.twitter.com/1.1/direct_messages/events/new.json",
            value
        )?.json()?;
        Ok(res)
    }

    fn get_account_activity_subscription(
        &self,
        env_name: &str
    ) -> Result<serde_json::Value, TwapiError> {
        let mut res = self.get(
            format!("https://api.twitter.com/1.1/account_activity/all/{}/subscriptions.json", env_name).as_str(),
            &vec![]
        )?;
        if res.status().as_u16() == 204 {
            Ok(serde_json::from_str("{}")?)
        } else {
            Ok(res.json()?)
        }
    }

    fn get_direct_messages_welcome_messages_list(
        &self
    ) -> Result<serde_json::Value, TwapiError> {
        let res = self.get(
            "https://api.twitter.com/1.1/direct_messages/welcome_messages/list.json",
            &vec![]
        )?.json()?;
        Ok(res)
    }

    fn get_direct_messages_welcome_messages_show(
        &self,
        id: &str
    ) -> Result<serde_json::Value, TwapiError> {
        let res = self.get(
            "https://api.twitter.com/1.1/direct_messages/welcome_messages/show.json",
            &vec![("id", id)]
        )?.json()?;
        Ok(res)
    }

    fn post_direct_messages_welcome_messages_new(
        &self,
        value: &serde_json::Value
    ) -> Result<serde_json::Value, TwapiError> {
        let res = self.json(
            "https://api.twitter.com/1.1/direct_messages/welcome_messages/new.json",
            value
        )?.json()?;
        Ok(res)
    }

    fn delete_direct_messages_welcome_messages_destroy(
        &self,
        id: &str
    ) -> Result<serde_json::Value, TwapiError> {
        let mut res = self.delete(
            "https://api.twitter.com/1.1/direct_messages/welcome_messages/destroy.json",
            &vec![("id", id)]
        )?;
        if res.status().as_u16() == 204 {
            Ok(serde_json::from_str("{}")?)
        } else {
            Ok(res.json()?)
        }
    }

    fn post_media_upload(
        &self,
        file: &str,
        additional_owners: Option<String>
    ) -> Result<serde_json::Value, TwapiError> {
        let form = reqwest::multipart::Form::new()
            .file("media", file)?;
        let form = if let Some(additional_owners) = additional_owners {
            form.text("additional_owners", additional_owners)
        } else {
            form
        };
        let res = self.multipart(
            "https://upload.twitter.com/1.1/media/upload.json",
            form
        )?.json()?;
        Ok(res)
    }

    fn post_media_upload_chunk(
        &self,
        file: &str,
        media_type: &str,
        media_category: &str,
        additional_owners: Option<String>
    ) -> Result<serde_json::Value, TwapiError> {
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
            let res_init: serde_json::Value = self.multipart(
                "https://upload.twitter.com/1.1/media/upload.json",
                form
            )?.json()?;
            match res_init.get("media_id_string") {
                Some(media_id) => String::from(media_id.as_str().unwrap()),
                None => return Ok(res_init.clone())
            }
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
            if response.status().as_u16() > 299 {
                let res = response.json()?;
                return Ok(res);
            }
        }

        let form = reqwest::multipart::Form::new()
            .text("command", "FINALIZE")
            .text("media_id", media_id);

        let res = self.multipart(
            "https://upload.twitter.com/1.1/media/upload.json",
            form
        )?.json()?;
        Ok(res)
    }
}

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