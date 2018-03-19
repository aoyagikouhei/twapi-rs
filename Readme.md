# twapi-rs

A simple Twitter library. This is easy for customize.

[Documentation](https://docs.rs/twapi)

## Description

*twapi-rs** is is a simple Twitter library. This is easy for customize.

## Features
- Application Only Authentication
- User Authentication
- JSON support(dm_event, welcome_message, media_metadata)
- Oauth1.0 Authentication
- Media Upload
- Add OAuth Web Application Example
- Any Twitter API Exucutable (maybe...)

## Examples

```rust
extern crate twapi;
extern crate serde_json;

use twapi::Twapi;
fn main() {
    // Application Only Authentication Sample
    let consumer_key = "xxx";
    let consumer_secret = "xxx";
    let applicaiton_auth = twapi::ApplicationAuth::new_with_consumer(
        consumer_key,
        consumer_secret
    );
    let res = applicaiton_auth.get_search_tweets(
        &vec![("q", "新宿"), ("count", "2")]
    ).unwrap();
    println!("{:?}", res);

    // Custmize Sample. Any API Executable!
    let res: serde_json::Value = applicaiton_auth.get(
        "https://api.twitter.com/1.1/statuses/user_timeline.json",
        &vec![("screen_name", "aoyagikouhei"), ("count", "2")]
    ).unwrap().json().unwrap();
    println!("{:?}", res);

    // JSON Sample
    let user_auth = twapi::UserAuth::new(
        "xxx",
        "xxx",
        "xxx",
        "xxx"
    );
    let data = r#"{
        "event": {
            "type": "message_create",
            "message_create": {
                "target": {
                    "recipient_id": "19522946"
                },
                "message_data": {
                    "text": "予定表〜①ﾊﾝｶｸだ!"
                }
            }
        }
    }"#;
    let v : serde_json::Value = serde_json::from_str(data).unwrap();
    let res = user_auth.post_direct_messages_events_new(&v);
    println!("{:?}", res);

    // Media Upload
    let res = user_auth.post_media_upload_chunk("test.mp4", "video/mp4", "tweet_video", None);
    println!("{:?}", res);
}
```