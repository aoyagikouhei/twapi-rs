extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate twapi;

use hyper::{Response, StatusCode};
use hyper::header::{Location};
use gotham::http::response::create_response;
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::{FromState, State};
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::middleware::session::{NewSessionMiddleware, SessionData};
use std::env;

#[derive(Clone, StateData)]
pub struct OauthData {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub callback_uri: String,
}

#[derive(Clone, NewMiddleware)]
pub struct OauthMiddleware {
    pub oauth_data: OauthData,
}

#[derive(Clone, Deserialize, Serialize, StateData)]
struct RequestToken {
    oauth_token: String,
    oauth_token_secret: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct CallbackQueryStringExtractor {
    oauth_verifier: String,
}

impl Middleware for OauthMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture>,
    {
        state.put(self.oauth_data.clone());
        chain(state)
    }
}

pub fn request_token_handler(mut state: State) -> (State, Response) {
    let (consumer_key, consumer_secret, callback_uri): (String, String, String) = {
        let oauth_data = OauthData::borrow_mut_from(&mut state);
        (
            oauth_data.consumer_key.clone(), 
            oauth_data.consumer_secret.clone(),
            oauth_data.callback_uri.clone(),
        )
    };
    let (oauth_token, oauth_token_secret, uri) = twapi::oauth1::request_token(
        &consumer_key, &consumer_secret, &callback_uri, None).unwrap();
    {
        let request_token: &mut Option<RequestToken> =
            SessionData::<Option<RequestToken>>::borrow_mut_from(&mut state);
        *request_token = Some(RequestToken {
            oauth_token: oauth_token,
            oauth_token_secret: oauth_token_secret,
        });
    }
    let mut res = create_response(
        &state, 
        StatusCode::TemporaryRedirect, 
        None,
    );
    res.headers_mut().set(Location::new(uri));
    (state, res)
}

pub fn callback_handler(mut state: State) -> (State ,Response) {
    let (consumer_key, consumer_secret): (String, String) = {
        let oauth_data = OauthData::borrow_mut_from(&mut state);
        (
            oauth_data.consumer_key.clone(), 
            oauth_data.consumer_secret.clone(),
        )
    };
    let maybe_request_token = {
        let request_token: &Option<RequestToken> = SessionData::<Option<RequestToken>>::borrow_from(&state);
        request_token.clone()
    };
    let query_param = CallbackQueryStringExtractor::take_from(&mut state);
    let body = match &maybe_request_token {
        &Some(ref request_token) => {
            let (oauth_token, oauth_token_secret, user_id, screen_name) = twapi::oauth1::access_token(
                &consumer_key, 
                &consumer_secret, 
                &request_token.oauth_token, 
                &request_token.oauth_token_secret, 
                &query_param.oauth_verifier).unwrap();
            format!("{},{},{},{}", oauth_token, oauth_token_secret, user_id, screen_name)
        },
        &None => String::from("ng"),
    };
    let res = create_response(
        &state, 
        StatusCode::Ok, 
        Some((body.into_bytes(), mime::TEXT_PLAIN_UTF_8)),
    );
    (state, res)
}

fn router(consumer_key: &str, consumer_secret: &str, callback_uri: &str) -> Router {
    let oauth_middleware = OauthMiddleware{
        oauth_data: OauthData {
            consumer_key: String::from(consumer_key),
            consumer_secret: String::from(consumer_secret),
            callback_uri: String::from(callback_uri),
        },
    };
    let session_middleware = NewSessionMiddleware::default()
        .with_session_type::<Option<RequestToken>>()
        .insecure();
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(oauth_middleware)
            .add(session_middleware)
            .build()
    );
    build_router(chain, pipelines, |route| {
        route
            .get("/request_token")
            .to(request_token_handler);
        route
            .get("/callback")
            .with_query_string_extractor::<CallbackQueryStringExtractor>()
            .to(callback_handler);
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router(&args[1], &args[2], &args[3]))
}
