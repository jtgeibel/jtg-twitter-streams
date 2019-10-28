use super::{plot, KeywordScore, KEYWORDS};

use std::time::Instant;

use futures::future;
use log::Level::Trace;
use log::{error, log_enabled, trace};
use tokio_executor::threadpool;
use tokio_sync::Lock;

/// Application start time and a counter to determine when a new interval has occurred
pub struct TickTracker {
    /// Starting time of the application
    started_at: Instant,
    /// Count of seconds since the app was started
    seconds_count: u64,
}

impl TickTracker {
    pub fn new() -> Self {
        TickTracker {
            started_at: Instant::now(),
            seconds_count: 0,
        }
    }

    pub fn runtime(&self) -> u64 {
        (Instant::now() - self.started_at).as_secs()
    }
}

pub async fn process_twitter_message(
    json: string::String<bytes::Bytes>,
    // If multiple locks are held at once, they should be locked in parameter order to avoid a deadlock
    mut tick_tracker: Lock<TickTracker>,
    mut current_scores: Lock<Vec<KeywordScore>>,
    mut chart: Lock<plot::Chart>,
    mut tweet_count: Lock<u32>,
) -> Result<(), twitter_stream::error::Error> {
    // Save data at 1 second intervals
    let mut tick_tracker = tick_tracker.lock().await;
    let runtime = tick_tracker.runtime();
    if runtime > tick_tracker.seconds_count {
        // FIXME(JTG): If no messages come in within a 1 second interval, then no points will be
        // plotted for that interval

        let current_scores = current_scores.lock().await;
        let mut chart = chart.lock().await;
        let current_scores: Vec<_> = current_scores.iter().map(KeywordScore::average).collect();
        for (idx, score) in current_scores.iter().enumerate() {
            // FIXME(JTG): Old data is never removed, this grows without bound
            chart.push(idx, *score);
        }

        // FIXME(JTG): If chart drawing becomes slow it will delay releasing lock on seconds_count, stalling other tasks
        future::poll_fn(|_| threadpool::blocking(|| chart.plot_and_save().unwrap()))
            .await
            .unwrap();
        tick_tracker.seconds_count = runtime;
    }
    drop(tick_tracker);

    let result = future::poll_fn(|_| threadpool::blocking(|| process(&json)))
        .await
        .unwrap();
    let (tweet, score) = match result {
        Ok(s) => s,
        Err(ProcessError::NoTweet(value)) => {
            error!("NoTweet: {}", value);
            return Ok(());
        }
        Err(_) => return Ok(()),
    };

    // Determine the keyword(s) present (could match more than one)
    // The whole blob from twitter is scanned, because many times the keyword is not in
    // the tweet itself
    let mut scores = current_scores.lock().await; // FIXME(JTG): Do less work while holding this lock
    for (i, keyword) in KEYWORDS.iter().enumerate() {
        if json.contains(keyword) {
            scores[i].add(score);
        }
    }

    let mut tweet_count = tweet_count.lock().await;
    *tweet_count += 1;
    if log_enabled!(Trace) {
        let average_scores: Vec<_> = scores.iter().map(KeywordScore::average).collect();
        trace!(
            "{}: {} - {:?}; tweet: {}",
            *tweet_count,
            score,
            average_scores,
            tweet
        );
    }

    Ok(())
}

/// Errors that may be encountered when deserializing and analyzing a tweet
enum ProcessError {
    InvalidJson(serde_json::Error),
    NoTweet(serde_json::Value),
}

impl From<serde_json::Error> for ProcessError {
    fn from(e: serde_json::Error) -> Self {
        ProcessError::InvalidJson(e)
    }
}

/// Decode the message and run sentiment analysis if a tweet is present
///
/// # Blocking
///
/// This function may be computationally intensive.
fn process(json: &str) -> Result<(String, f32), ProcessError> {
    let json: serde_json::Value = serde_json::from_str(json)?;
    if let Some(tweet) = json.get("text") {
        let sentiment = sentiment::analyze(tweet.to_string());
        Ok((tweet.to_string(), sentiment.score))
    } else {
        Err(ProcessError::NoTweet(json))
    }
}
