#![deny(warnings, clippy::all)]

use std::time::Instant;

use hyper::service::service_fn;
use hyper::Server;
use log::Level::Trace;
use log::{debug, error, log_enabled, trace};
use twitter_stream::rt::{self, Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

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
    dotenv::dotenv().expect("Failed to load .env file");

    let consumer_key = dotenv::var("TWITTER_CONSUMER_KEY")
        .expect("Environment variable TWITTER_CONSUMER_KEY is not set");
    let consumer_secret = dotenv::var("TWITTER_CONSUMER_SECRET")
        .expect("Environment variable TWITTER_CONSUMER_SECRET is not set");
    let token_key = dotenv::var("TWITTER_TOKEN_KEY")
        .expect("Environment variable TWITTER_TOKEN_KEY is not set");
    let token_secret = dotenv::var("TWITTER_TOKEN_SECRET")
        .expect("Environment variable TWITTER_TOKEN_SECRET is not set");

    let token = Token::new(consumer_key, consumer_secret, token_key, token_secret);
    let keywords = KEYWORDS.join(",");

    // Initialize a Vec<f32> for each keyword
    let mut chart: Vec<Vec<f32>> = Vec::new();
    chart.resize_with(KEYWORDS.len(), Default::default);

    // Data used by the stream handler
    let mut count = 0u32;
    let mut accum: Vec<Score> = Vec::new();
    accum.resize_with(KEYWORDS.len(), Default::default);

    // Draw initial empty plot
    plot::draw_chart(&chart).unwrap();

    let started_at = Instant::now();
    let mut seconds_count = 0u64;

    let twitter_stream = TwitterStreamBuilder::filter(token)
        .stall_warnings(true)
        // Since twitter is rate limiting, may as well focus on tweets we can analyze
        .language("en")
        .track(Some(&*keywords))
        .listen()
        .unwrap()
        .flatten_stream()
        .for_each(move |json| {
            // Save data at 1 second intervals
            let runtime = (Instant::now() - started_at).as_secs();
            if runtime > seconds_count {
                // FIXME(JTG): If no messages come in within a second interval, then no points will be
                // plotted for that interval

                let current_scores: Vec<_> = accum.iter().map(Score::average).collect();
                for (idx, score) in current_scores.iter().enumerate() {
                    // FIXME(JTG): Old data is never removed, this grows without bound
                    chart[idx].push(*score);
                }

                plot::draw_chart(&chart).unwrap();
                seconds_count = runtime;
            }

            let (message, score) = match process(&json) {
                Ok(s) => s,
                Err(ProcessError::NoTweet(value)) => {
                    error!("NoTweet: {}", value);
                    let current_scores: Vec<_> = accum.iter().map(Score::average).collect();
                    debug!("Message count: {}, Summary: {:?}", count, current_scores);
                    return Ok(());
                }
                Err(_) => return Ok(()),
            };

            count += 1;

            // Determine the keyword(s) present (could match more than one)
            // The whole blob from twitter is scanned, because many times the keyword is not in
            // the message itself
            for (i, keyword) in KEYWORDS.iter().enumerate() {
                if json.contains(keyword) {
                    accum[i].add(score);
                }
            }

            if log_enabled!(Trace) {
                let current_scores: Vec<_> = accum.iter().map(Score::average).collect();
                trace!(
                    "{}: {} - {:?}; tweet: {}",
                    count,
                    score,
                    current_scores,
                    message
                );
            }

            Ok(())
        })
        .map_err(|e| error!("Twitter stream error: {}", e));

    let port = dotenv::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8888);
    let addr = ([127, 0, 0, 1], port).into();

    let service = || {
        service_fn(server::handle_request)
    };

    let server = Server::bind(&addr)
        .serve(service)
        .map_err(|e| error!("Server error: {}", e));

    let future = twitter_stream.select2(server).map(|_| ()).map_err(|_| ());
    rt::run(future);
}

#[derive(Clone, Debug, Default)]
struct Score {
    score_sum: f32,
    message_count: u32,
}

impl Score {
    fn add(&mut self, score: f32) {
        self.score_sum += score;
        self.message_count += 1;
    }

    fn average(&self) -> f32 {
        if self.message_count == 0 {
            return 0.0;
        };
        self.score_sum / (self.message_count as f32)
    }
}

enum ProcessError {
    InvalidJson(serde_json::Error),
    NoTweet(serde_json::Value),
}

impl From<serde_json::Error> for ProcessError {
    fn from(e: serde_json::Error) -> Self {
        ProcessError::InvalidJson(e)
    }
}

fn process(json: &str) -> Result<(String, f32), ProcessError> {
    let json: serde_json::Value = serde_json::from_str(json)?;
    if let Some(message) = json.get("text") {
        let sentiment = sentiment::analyze(message.to_string());
        Ok((message.to_string(), sentiment.score))
    } else {
        Err(ProcessError::NoTweet(json))
    }
}
