//! A simple Twitter library. This is easy for customize.
use async_trait::async_trait;
use std::{
    io::{BufReader, Cursor, Read},
    time,
};
use twapi_reqwest::reqwest::{
    multipart::{Form, Part},
    Error, Response,
};

pub mod oauth1;
pub mod oauth2;

#[cfg(feature = "account-activity")]
pub mod account_activity;

type TResult = Result<Response, Error>;

/// Response from Twitter API
#[derive(Debug)]
pub struct TwapiResponse {
    pub status_code: u16,
    pub json: Option<serde_json::Value>,
}

impl TwapiResponse {
    pub async fn new(response: Response) -> TwapiResponse {
        let status_code = response.status().as_u16();
        let json = match response.json().await {
            Ok(json) => Some(json),
            Err(_) => None,
        };
        TwapiResponse {
            status_code: status_code,
            json: json,
        }
    }

    pub fn is_success(&self) -> bool {
        200 <= self.status_code && self.status_code < 300
    }

    pub fn copy_json_value(&self) -> Option<serde_json::Value> {
        match &self.json {
            Some(ref json) => Some(json.clone()),
            None => None,
        }
    }
}

/// Error in twapi library
#[derive(Debug)]
pub enum TwapiError {
    Connection(Error),
    IO(std::io::Error),
    Token((u16, String)),
    UrlParse(url::ParseError),
    NotExists,
}

impl From<Error> for TwapiError {
    fn from(err: Error) -> TwapiError {
        TwapiError::Connection(err)
    }
}

impl From<std::io::Error> for TwapiError {
    fn from(err: std::io::Error) -> TwapiError {
        TwapiError::IO(err)
    }
}

impl From<url::ParseError> for TwapiError {
    fn from(err: url::ParseError) -> TwapiError {
        TwapiError::UrlParse(err)
    }
}

fn make_account_activity_uri(
    command_type: &str,
    env_name: Option<&str>,
    file_name: Option<&str>,
) -> String {
    let prefix = match env_name {
        Some(env_name) => format!(
            "https://api.twitter.com/1.1/account_activity/all/{}/{}",
            env_name, command_type
        ),
        None => format!(
            "https://api.twitter.com/1.1/account_activity/{}",
            command_type
        ),
    };
    match file_name {
        Some(file_name) => format!("{}/{}.json", prefix, file_name),
        None => format!("{}.json", prefix),
    }
}

/// Access to Twitter API
#[async_trait]
pub trait Twapi {
    async fn get(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult;
    async fn post(
        &self,
        uri: &str,
        query_options: &Vec<(&str, &str)>,
        form_options: &Vec<(&str, &str)>,
    ) -> TResult;

    async fn multipart(&self, uri: &str, query_options: &Vec<(&str, &str)>, form: Form) -> TResult;

    async fn put(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult;

    async fn delete(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult;

    async fn json(
        &self,
        uri: &str,
        query_options: &Vec<(&str, &str)>,
        json: &serde_json::Value,
    ) -> TResult;

    async fn get_verify_credentials(
        &self,
        params: &Vec<(&str, &str)>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                "https://api.twitter.com/1.1/account/verify_credentials.json",
                params,
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_search_tweets(
        &self,
        params: &Vec<(&str, &str)>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get("https://api.twitter.com/1.1/search/tweets.json", params)
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn post_statuses_update(
        &self,
        params: &Vec<(&str, &str)>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .post(
                "https://api.twitter.com/1.1/statuses/update.json",
                &vec![],
                params,
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn post_direct_messages_events_new(
        &self,
        value: &serde_json::Value,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .json(
                "https://api.twitter.com/1.1/direct_messages/events/new.json",
                &vec![],
                value,
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_account_activity_subscription(
        &self,
        env_name: &str,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                format!(
                    "https://api.twitter.com/1.1/account_activity/all/{}/subscriptions.json",
                    env_name
                )
                .as_str(),
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_direct_messages_welcome_messages_list(&self) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                "https://api.twitter.com/1.1/direct_messages/welcome_messages/list.json",
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_direct_messages_welcome_messages_show(
        &self,
        id: &str,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                "https://api.twitter.com/1.1/direct_messages/welcome_messages/show.json",
                &vec![("id", id)],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn post_direct_messages_welcome_messages_new(
        &self,
        value: &serde_json::Value,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .json(
                "https://api.twitter.com/1.1/direct_messages/welcome_messages/new.json",
                &vec![],
                value,
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn delete_direct_messages_welcome_messages_destroy(
        &self,
        id: &str,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .delete(
                "https://api.twitter.com/1.1/direct_messages/welcome_messages/destroy.json",
                &vec![("id", id)],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_media_upload(&self, media_id: &str) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                "https://upload.twitter.com/1.1/media/upload.json",
                &vec![("command", "STATUS"), ("media_id", media_id)],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_media_upload_until_succeeded(
        &self,
        media_id: &str,
    ) -> Result<TwapiResponse, TwapiError> {
        loop {
            let check_after_secs = {
                let result = self.get_media_upload(&media_id).await?;
                if !result.is_success() {
                    return Ok(result);
                }
                let json = result.copy_json_value().unwrap();
                let processing_info = json.get("processing_info").unwrap();
                let state = String::from(processing_info.get("state").unwrap().as_str().unwrap());
                if state == "succeeded" || state == "failed" {
                    return Ok(result);
                }
                processing_info
                    .get("check_after_secs")
                    .unwrap()
                    .as_u64()
                    .unwrap()
            };
            tokio::time::delay_for(time::Duration::new(check_after_secs, 0)).await;
        }
    }

    async fn post_media_upload(
        &self,
        file: &str,
        additional_owners: Option<String>,
    ) -> Result<TwapiResponse, TwapiError> {
        let metadata = std::fs::metadata(file).unwrap();
        let file_size = metadata.len();
        let f = std::fs::File::open(file).unwrap();
        let mut cursor = Cursor::new(vec![0; file_size as usize]);
        let mut reader = BufReader::new(f);
        reader.read(cursor.get_mut()).unwrap();

        let part = Part::bytes(cursor.into_inner());
        let form = Form::new().part("media", part);
        let form = if let Some(additional_owners) = additional_owners {
            form.text("additional_owners", additional_owners)
        } else {
            form
        };
        let res = self
            .multipart(
                "https://upload.twitter.com/1.1/media/upload.json",
                &vec![],
                form,
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn post_media_upload_chunk(
        &self,
        file: &str,
        media_type: &str,
        media_category: &str,
        additional_owners: Option<String>,
    ) -> Result<TwapiResponse, TwapiError> {
        let metadata = std::fs::metadata(file)?;
        let file_size = metadata.len();
        let form = Form::new()
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
            let response = self
                .multipart(
                    "https://upload.twitter.com/1.1/media/upload.json",
                    &vec![],
                    form,
                )
                .await?;
            let result = TwapiResponse::new(response).await;
            if !result.is_success() {
                return Ok(result);
            }
            String::from(
                result
                    .json
                    .unwrap()
                    .get("media_id_string")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
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
            let form = Form::new()
                .text("command", "APPEND")
                .text("media_id", media_id.clone())
                .text("segment_index", segment_index.to_string())
                .part("media", Part::bytes(cursor.into_inner()));

            let response = self
                .multipart(
                    "https://upload.twitter.com/1.1/media/upload.json",
                    &vec![],
                    form,
                )
                .await?;
            segment_index = segment_index + 1;
            let result = TwapiResponse::new(response).await;
            if !result.is_success() {
                return Ok(result);
            }
        }

        let form = Form::new()
            .text("command", "FINALIZE")
            .text("media_id", media_id.clone());

        let response = self
            .multipart(
                "https://upload.twitter.com/1.1/media/upload.json",
                &vec![],
                form,
            )
            .await?;
        let result = TwapiResponse::new(response).await;
        if !result.is_success() {
            return Ok(result);
        }
        let json = result.copy_json_value().unwrap();
        let processing_info = json.get("processing_info");
        if processing_info == None {
            Ok(result)
        } else {
            // check only processing_info included.
            // if not included call then you get "Invalid mediaId.".
            self.get_media_upload_until_succeeded(&media_id).await
        }
    }

    async fn post_media_metadata_create(
        &self,
        value: &serde_json::Value,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .json(
                "https://upload.twitter.com/1.1/media/metadata/create.json",
                &vec![],
                value,
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn post_account_activity_webhooks(
        &self,
        uri: &str,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .post(
                &make_account_activity_uri("webhooks", env_name, None),
                &vec![("url", uri)],
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_account_activity_webhooks(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                &make_account_activity_uri("webhooks", env_name, None),
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    // Always Fails in Standard(Beta)
    async fn put_account_activity_webhooks(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .put(
                &make_account_activity_uri("webhooks", env_name, None),
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn delete_account_activity_webhooks(
        &self,
        webhook_id: &str,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .delete(
                &make_account_activity_uri("webhooks", env_name, Some(webhook_id)),
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn post_account_activity_subscriptions(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .post(
                &make_account_activity_uri("subscriptions", env_name, None),
                &vec![],
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_account_activity_all_count(&self) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                "https://api.twitter.com/1.1/account_activity/all/count.json",
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_account_activity_subscriptions(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                &make_account_activity_uri("subscriptions", env_name, None),
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn get_account_activity_subscriptions_list(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .get(
                &make_account_activity_uri("subscriptions", env_name, Some("list")),
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }

    async fn delete_account_activity_subscriptions(
        &self,
        env_name: Option<&str>,
    ) -> Result<TwapiResponse, TwapiError> {
        let res = self
            .delete(
                &make_account_activity_uri("subscriptions", env_name, None),
                &vec![],
            )
            .await?;
        Ok(TwapiResponse::new(res).await)
    }
}

/// Application Only Authenticaiton by oauth2
pub struct ApplicationAuth {
    bearer_token: String,
}

impl ApplicationAuth {
    pub fn new(bearer_token: &str) -> ApplicationAuth {
        ApplicationAuth {
            bearer_token: String::from(bearer_token),
        }
    }
}

#[async_trait]
impl Twapi for ApplicationAuth {
    async fn get(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult {
        twapi_reqwest::v2::get(uri, query_options, &self.bearer_token).await
    }

    async fn post(
        &self,
        uri: &str,
        query_options: &Vec<(&str, &str)>,
        form_options: &Vec<(&str, &str)>,
    ) -> TResult {
        twapi_reqwest::v2::post(uri, query_options, form_options, &self.bearer_token).await
    }

    async fn multipart(&self, uri: &str, query_options: &Vec<(&str, &str)>, form: Form) -> TResult {
        twapi_reqwest::v2::multipart(uri, query_options, form, &self.bearer_token).await
    }

    async fn put(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult {
        twapi_reqwest::v2::put(uri, query_options, &self.bearer_token).await
    }

    async fn delete(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult {
        twapi_reqwest::v2::delete(uri, query_options, &self.bearer_token).await
    }

    async fn json(
        &self,
        uri: &str,
        query_options: &Vec<(&str, &str)>,
        json: &serde_json::Value,
    ) -> TResult {
        twapi_reqwest::v2::json(uri, query_options, json, &self.bearer_token).await
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
    pub fn new(
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

#[async_trait]
impl Twapi for UserAuth {
    async fn get(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult {
        twapi_reqwest::v1::get(
            uri,
            query_options,
            &self.consumer_key,
            &self.consumer_secret,
            &self.access_token,
            &self.access_token_secret,
        )
        .await
    }

    async fn post(
        &self,
        uri: &str,
        query_options: &Vec<(&str, &str)>,
        form_options: &Vec<(&str, &str)>,
    ) -> TResult {
        twapi_reqwest::v1::post(
            uri,
            query_options,
            form_options,
            &self.consumer_key,
            &self.consumer_secret,
            &self.access_token,
            &self.access_token_secret,
        )
        .await
    }

    async fn multipart(&self, uri: &str, query_options: &Vec<(&str, &str)>, form: Form) -> TResult {
        twapi_reqwest::v1::multipart(
            uri,
            query_options,
            form,
            &self.consumer_key,
            &self.consumer_secret,
            &self.access_token,
            &self.access_token_secret,
        )
        .await
    }

    async fn put(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult {
        twapi_reqwest::v1::put(
            uri,
            query_options,
            &self.consumer_key,
            &self.consumer_secret,
            &self.access_token,
            &self.access_token_secret,
        )
        .await
    }

    async fn delete(&self, uri: &str, query_options: &Vec<(&str, &str)>) -> TResult {
        twapi_reqwest::v1::delete(
            uri,
            query_options,
            &self.consumer_key,
            &self.consumer_secret,
            &self.access_token,
            &self.access_token_secret,
        )
        .await
    }

    async fn json(
        &self,
        uri: &str,
        query_options: &Vec<(&str, &str)>,
        json: &serde_json::Value,
    ) -> TResult {
        twapi_reqwest::v1::json(
            uri,
            query_options,
            json,
            &self.consumer_key,
            &self.consumer_secret,
            &self.access_token,
            &self.access_token_secret,
        )
        .await
    }
}
