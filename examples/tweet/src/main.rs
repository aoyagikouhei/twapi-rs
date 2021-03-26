use twapi::Twapi;
use std::env;

#[tokio::main]
async fn main() {
    let consumer_key = env::var("CONSUMER_KEY").unwrap();
    let consumer_secret = env::var("CONSUMER_SECRET").unwrap();
    let access_key = env::var("ACCESS_KEY").unwrap();
    let access_secret = env::var("ACCESS_SECRET").unwrap();
    let user_auth = twapi::UserAuth::new(
        &consumer_key,
        &consumer_secret,
        &access_key,
        &access_secret,
    );
    let res = user_auth.post_statuses_update(&vec![("status", "!\"'#$%&\\()+,/:;<=>?@[\\]^{|}~;-._* 全部4`")]).await;
    //let res = user_auth.get_search_tweets(&vec![("q", "*abc"), ("count", "2")]).await;
    println!("{:?}", res);
}
