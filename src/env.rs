//! Helpers for accessing environment variables

use std::net::SocketAddr;

use twitter_stream::Token;

/// Generate a Twitter client token from keys and secrets supplied via the environment
///
/// # Panics
///
/// This function will panic if any of the necessary keys or secrets are missing.
pub fn get_client_token() -> Token {
    let consumer_key = dotenv::var("TWITTER_CONSUMER_KEY")
        .expect("Environment variable TWITTER_CONSUMER_KEY is not set");
    let consumer_secret = dotenv::var("TWITTER_CONSUMER_SECRET")
        .expect("Environment variable TWITTER_CONSUMER_SECRET is not set");
    let token_key = dotenv::var("TWITTER_TOKEN_KEY")
        .expect("Environment variable TWITTER_TOKEN_KEY is not set");
    let token_secret = dotenv::var("TWITTER_TOKEN_SECRET")
        .expect("Environment variable TWITTER_TOKEN_SECRET is not set");

    Token::new(consumer_key, consumer_secret, token_key, token_secret)
}

/// Pick a listening address and port for the web server based on environment variables
///
/// The default is to use a development environment listending on port 8888.  These can be
/// overwridden by setting the `APP_ENV` and `PORT` variables.
///
/// In development the server will bind only to the localhost address.  In production the server
/// will bind to all IP addresses.
pub fn get_server_addr() -> SocketAddr {
    let port = dotenv::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8888);

    if let Some("production") = dotenv::var("APP_ENV").ok().as_ref().map(String::as_str) {
        println!(
            "Running in production mode, binding to port {} on all addresses",
            port
        );
        ([0, 0, 0, 0], port).into()
    } else {
        println!(
            "Running in development mode, binding to port {} on localhost only",
            port
        );
        ([127, 0, 0, 1], port).into()
    }
}
