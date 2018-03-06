//! A simple Twitter library. This is easy for customize.
extern crate base64;
extern crate reqwest;
extern crate serde_json;

pub mod oauth1;
pub mod oauth2;

#[derive(Debug)]
pub enum TwapiError {
    Connection(reqwest::Error),
    JSON(serde_json::Error),
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

pub trait Twapi {
    fn get(&self, uri: &str, options: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError>;
    fn post(&self, uri: &str, options: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError>;
    fn delete(&self, uri: &str, options: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError>;
    fn json(&self, uri: &str, value: &serde_json::Value) -> Result<reqwest::Response, TwapiError>;

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

    fn delete(&self, _: &str, _: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError> {
        Err(TwapiError::NotExists)
    }

    fn post(&self, _: &str, _: &Vec<(&str, &str)>) -> Result<reqwest::Response, TwapiError> {
        Err(TwapiError::NotExists)
    }

    fn json(&self, _: &str, _: &serde_json::Value) -> Result<reqwest::Response, TwapiError> {
        Err(TwapiError::NotExists)
    }
}