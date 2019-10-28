#![deny(warnings, clippy::all)]

use futures::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::error;

mod client;
mod env;
mod plot;
mod server;

/// Keywords to be tracked by the application
static KEYWORDS: &[&str] = &[
    "twitter",
    "facebook",
    "google",
    "travel",
    "art",
    "music",
    "photography",
    "love",
    "fashion",
    "food",
];

fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    // Configure Twitter Streaming API "client task"
    let client_state = client::ClientState::init();
    let token = env::get_client_token();
    let keywords = KEYWORDS.join(",");
    let client = twitter_stream::Builder::filter(token)
        .stall_warnings(true)
        // Since twitter is rate limiting, may as well focus on tweets we can analyze
        .language("en")
        .track(Some(&*keywords))
        .listen()
        .unwrap()
        .try_flatten_stream()
        // FIXME(JTG): This stream is not actually being processed concurrently and its unclear why not
        // See the note in the architecture section of the README
        .try_for_each_concurrent(4, move |json| {
            client::process_twitter_message(json, client_state.clone())
        });

    // Configure the HTTP "server task"
    let make_service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(server::handle_request)) });
    let server = Server::bind(&env::get_server_addr()).serve(make_service);

    // Spawn the top-level client and server tasks
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(async {
        if let Err(e) = client.await {
            error!("Twitter stream error: {}", e);
        }
    });
    rt.spawn(async {
        if let Err(e) = server.await {
            error!("Web server error: {}", e);
        }
    });
    rt.shutdown_on_idle();
}
