#![deny(warnings, clippy::all)]

use log::Level::Trace;
use log::{debug, error, log_enabled, trace};
use twitter_stream::rt::{self, Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

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

    let mut count = 0;
    let mut accum: Vec<Score> = Vec::new();
    accum.resize_with(KEYWORDS.len(), Default::default);

    let twitter_stream = TwitterStreamBuilder::filter(token)
        .stall_warnings(true)
        // Since twitter is rate limiting, may as well focus on tweets we can analyze
        .language("en")
        .track(Some(&*keywords))
        .listen()
        .unwrap()
        .flatten_stream()
        .for_each(move |json| {
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
        .map_err(|e| println!("error: {}", e));

    rt::run(twitter_stream);
}

#[derive(Debug, Default)]
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
