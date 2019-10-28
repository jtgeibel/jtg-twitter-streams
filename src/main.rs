#![deny(warnings, clippy::all)]

use futures::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::error;
use tokio_sync::Lock;

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

/// A tally of the total score and tweet count for a keyword
#[derive(Clone, Debug, Default)]
pub struct KeywordScore {
    score_sum: f32,
    tweet_count: u32,
}

impl KeywordScore {
    fn add(&mut self, score: f32) {
        self.score_sum += score;
        self.tweet_count += 1;
    }

    fn average(&self) -> f32 {
        if self.tweet_count == 0 {
            return 0.0;
        };
        self.score_sum / (self.tweet_count as f32)
    }
}

fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    // Draw initial empty plot
    let chart = plot::Chart::new();
    chart.plot_and_save().unwrap();
    let chart = Lock::new(chart);

    // Track duration since program start to trigger new chart plots
    let tick_tracker = Lock::new(client::TickTracker::new());

    // Total tweet count, for diagnostics purposes
    let tweet_count = Lock::new(0u32);

    // An accumulated current score for each keyword
    let mut current_scores: Vec<KeywordScore> = Vec::new();
    current_scores.resize_with(KEYWORDS.len(), Default::default);
    let current_scores = Lock::new(current_scores);

    // Configure Twitter Streaming API "client task"
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
            let tick_tracker = tick_tracker.clone();
            let current_scores = current_scores.clone();
            let chart = chart.clone();
            let tweet_count = tweet_count.clone();
            client::process_twitter_message(json, tick_tracker, current_scores, chart, tweet_count)
        });

    // Configure the web "server task"
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
