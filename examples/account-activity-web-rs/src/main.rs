extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate twapi;

use futures::{Stream, Future};
use hyper::{Response, StatusCode, Body, Headers};
use gotham::http::response::create_response;
use gotham::handler::{IntoHandlerError, HandlerFuture};
use gotham::middleware::Middleware;
use gotham::state::{FromState, State};
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use std::env;

#[derive(StateData)]
struct ApplicationData {
    conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>,
    consumer_secret: String,
}

#[derive(Clone, NewMiddleware)]
struct ApplicationMiddleware {
    consumer_secret: String,
    pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>
}

impl Middleware for ApplicationMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture>,
    {
        state.put(ApplicationData{
            conn: self.pool.get().unwrap(),
            consumer_secret: self.consumer_secret.clone(),
        });
        chain(state)
    }
}

impl std::panic::RefUnwindSafe for ApplicationMiddleware {}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QueryStringExtractor {
    crc_token: String,
}

pub fn get_handler(mut state: State) -> (State, Response) {
    let headers = Headers::take_from(&mut state);
    let ua = String::from_utf8_lossy(headers.get_raw("user-agent").unwrap().one().unwrap());
    println!("{}", ua);
    // "X-Real-IP"
    // "X-Twitter-Webhooks-Signature"

    let consumer_secret: String = {
        let application_data = ApplicationData::borrow_mut_from(&mut state);
         application_data.consumer_secret.clone()
    };
    let query_param = QueryStringExtractor::take_from(&mut state);
    let calced_crc = twapi::account_activity::make_crc_token_response(&consumer_secret, &query_param.crc_token);
    let res = create_response(
        &state, 
        StatusCode::Ok, 
        Some((calced_crc.into_bytes(), mime::APPLICATION_JSON)),
    );
    (state, res)
}

pub fn post_handler(mut state: State) -> Box<HandlerFuture> {
    let f = Body::take_from(&mut state)
        .concat2()
        .then(|full_body| match full_body {
            Ok(valid_body) => {
                let headers = Headers::take_from(&mut state);
                let ip = String::from_utf8_lossy(headers.get_raw("X-Real-IP").unwrap().one().unwrap());
                let sign = String::from_utf8_lossy(headers.get_raw("X-Twitter-Webhooks-Signature").unwrap().one().unwrap());                

                let application_data = ApplicationData::take_from(&mut state);
                let body_content = String::from_utf8(valid_body.to_vec()).unwrap();

                let calced_sign = twapi::account_activity::calc_hmac(&application_data.consumer_secret, &body_content);
                let merged = format!("{},{}", sign, calced_sign);

                let json_value: serde_json::Value = serde_json::from_str(&body_content).unwrap();
                application_data.conn.execute(
                    "INSERT INTO test (data, headers, ip) VALUES ($1, $2, $3)", 
                    &[&json_value, &merged, &ip]).unwrap();
                let res = create_response(
                    &state, 
                    StatusCode::Ok, 
                    Some((String::from("ok").into_bytes(), mime::TEXT_PLAIN_UTF_8)),
                );
                futures::future::ok((state, res))
            }
            Err(e) => return futures::future::err((state, e.into_handler_error())),
        });
    
    Box::new(f)
}

fn router(pg_conn: &str, consumer_secret: &str) -> Router {
    let manager = r2d2_postgres::PostgresConnectionManager::new(
        pg_conn, 
        r2d2_postgres::TlsMode::None).unwrap();
    let application_middleware = ApplicationMiddleware{
        pool: r2d2::Pool::new(manager).unwrap(),
        consumer_secret: String::from(consumer_secret),
    };
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(application_middleware)
            .build()
    );
    build_router(chain, pipelines, |route| {
        route
            .get("/")
            .with_query_string_extractor::<QueryStringExtractor>()
            .to(get_handler);
        route
            .post("/")
            .to(post_handler);
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router(&args[1], &args[2]))
}
