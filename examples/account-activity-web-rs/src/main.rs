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
    pub consumer_secret: String,
}

#[derive(Clone, NewMiddleware)]
pub struct OauthMiddleware {
    pub oauth_data: OauthData,
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

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QueryStringExtractor {
    crc_token: String,
}

fn router(consumer_secret: &str) -> Router {
    let oauth_middleware = OauthMiddleware{
        oauth_data: OauthData {
            consumer_secret: String::from(consumer_secret),
        },
    };
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(oauth_middleware)
            .build()
    );
    build_router(chain, pipelines, |route| {
        route
            .get("/")
            .to(request_token_handler);
        route
            .post("/")
            .with_query_string_extractor::<CallbackQueryStringExtractor>()
            .to(callback_handler);
    })
}

fn main() {
    /*
    let args: Vec<String> = env::args().collect();
    let comsumer_secret = &args[1];
    let crc_token = &args[2];
    println!("{}", twapi::account_activity::make_crc_token_response(comsumer_secret, crc_token));
*/
    let args: Vec<String> = env::args().collect();
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router(&args[1]))
}
