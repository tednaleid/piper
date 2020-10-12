// derived from: https://github.com/gotham-rs/gotham/tree/master/examples/hello_world_until

use futures::prelude::*;
use std::pin::Pin;
use std::time::Duration;

use gotham::hyper::{body, Body, Response, StatusCode};

use gotham::handler::{HandlerFuture, HandlerResult};
use gotham::helpers::http::response::create_response;
use gotham::router::builder::DefineSingleRoute;
use gotham::router::builder::{build_simple_router, DrawRoutes};
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use serde_derive::Deserialize;

use tokio::time::delay_for;

type SleepFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QueryStringExtractor {
    seconds: u64,
}

/// Sneaky hack to make tests take less time. Nothing to see here ;-).
#[cfg(not(test))]
fn get_duration(seconds: u64) -> Duration {
    Duration::from_secs(seconds)
}

#[cfg(test)]
fn get_duration(seconds: u64) -> Duration {
    Duration::from_millis(seconds)
}

fn sleep(seconds: u64) -> SleepFuture {
    delay_for(get_duration(seconds)).boxed()
}

async fn sleep_handler(mut state: State) -> HandlerResult {
    let seconds = QueryStringExtractor::take_from(&mut state).seconds;
    let _ = sleep(seconds).await;

    let data = format!("slept {}s", seconds);
    let res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, data);
    Ok((state, res))
}

pub fn ping_pong_handler(state: State) -> (State, Response<Body>) {
    (
        state,
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("pong"))
            .unwrap(),
    )
}

fn echo_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = body::to_bytes(Body::take_from(&mut state)).then(|full_body| match full_body {
        Ok(body_content) => {
            let res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, body_content);
            future::ok((state, res))
        }
        Err(e) => future::err((state, e.into())),
    });

    f.boxed()
}

fn router() -> Router {
    build_simple_router(|route| {
        route
            .get("/sleep")
            .with_query_string_extractor::<QueryStringExtractor>()
            .to_async(sleep_handler);
        route.get("/ping").to(ping_pong_handler);
        route.post("/echo").to(echo_handler);
    })
}

pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}

#[cfg(test)]
mod tests {
    use gotham::test::TestServer;

    use super::*;

    fn assert_returns_ok(url_str: &str, expected_response: &str) {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server.client().get(url_str).perform().unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            &String::from_utf8(response.read_body().unwrap()).unwrap(),
            expected_response
        );
    }

    fn assert_post_returns_ok(url_str: &str, body_str: &'static str, expected_response: &str) {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .post(url_str, body_str, mime::TEXT_PLAIN)
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            &String::from_utf8(response.read_body().unwrap()).unwrap(),
            expected_response
        );
    }

    #[test]
    fn sleep_says_how_long_it_slept_for() {
        assert_returns_ok("http://localhost/sleep?seconds=2", "slept 2s");
    }

    #[test]
    fn ping_returns_pong() {
        assert_returns_ok("http://localhost/ping", "pong");
    }

    #[test]
    fn echo_returns_what_was_sent() {
        assert_post_returns_ok("http://localhost/echo", "echo", "echo");
    }
}
