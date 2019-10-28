use super::{plot::Chart, KEYWORDS};

use std::time::Instant;

use futures::future;
use log::Level::Trace;
use log::{error, log_enabled, trace};
use tokio::sync::Lock;
use tokio_executor::threadpool;

/// Locks that protect shared state between client tasks
///
/// If multiple locks are held at the same time, they should be locked in definition order to avoid
/// a deadlock.
#[derive(Clone)]
pub struct ClientState {
    /// Track duration since program start to trigger new chart plots
    tick_tracker: Lock<TickTracker>,
    /// An accumulated current score for each keyword
    current_scores: Lock<Vec<KeywordScore>>,
    /// Chart storing keyword `current_scores` for each tick interval
    chart: Lock<Chart>,
    /// Total tweet count, for diagnostics purposes
    tweet_count: Lock<u32>,
}

impl ClientState {
    pub fn init() -> Self {
        // Draw initial empty plot
        let chart = Chart::new();
        chart.plot_and_save().unwrap();
        let chart = Lock::new(chart);

        let tick_tracker = Lock::new(TickTracker::new());
        let tweet_count = Lock::new(0u32);

        let mut current_scores = Vec::new();
        current_scores.resize_with(KEYWORDS.len(), Default::default);
        let current_scores = Lock::new(current_scores);

        Self {
            tick_tracker,
            current_scores,
            chart,
            tweet_count,
        }
    }
}

/// A tally of the total score and tweet count for a keyword
#[derive(Debug, Default)]
pub struct KeywordScore {
    score_sum: f32,
    tweet_count: u32,
}

impl KeywordScore {
    fn add(&mut self, score: f32) {
        self.score_sum += score;
        self.tweet_count += 1;
    }

    pub fn average(&self) -> f32 {
        if self.tweet_count == 0 {
            return 0.0;
        };
        self.score_sum / (self.tweet_count as f32)
    }
}

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

/// Process a JSON message, updating shared client state to track scores and plot the analysis
pub async fn process_twitter_message(
    json: string::String<bytes::Bytes>,
    mut locks: ClientState,
) -> Result<(), twitter_stream::error::Error> {
    save_chart_if_new_tick(&mut locks).await;

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
    let mut scores = locks.current_scores.lock().await; // FIXME(JTG): Do less work while holding this lock
    for (i, keyword) in KEYWORDS.iter().enumerate() {
        if json.contains(keyword) {
            scores[i].add(score);
        }
    }

    let mut tweet_count = locks.tweet_count.lock().await;
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

/// Check if a new 1 second interval has passed, saving a new chart if so
async fn save_chart_if_new_tick(locks: &mut ClientState) {
    // FIXME(JTG): If no messages come in within a 1 second interval, then no points will be
    // plotted for that interval.  Look into moving this out of the client handler and scheduling
    // with tokio_timer.
    let mut tick_tracker = locks.tick_tracker.lock().await;
    let runtime = tick_tracker.runtime();
    if runtime > tick_tracker.seconds_count {
        let current_scores = locks.current_scores.lock().await;
        let mut chart = locks.chart.lock().await;
        // FIXME(JTG): Old data is never removed, this grows without bound
        chart.push(&current_scores);

        // FIXME(JTG): If chart drawing becomes slow it will delay releasing lock on tick_tracker, stalling other tasks
        future::poll_fn(|_| threadpool::blocking(|| chart.plot_and_save().unwrap()))
            .await
            .unwrap();
        tick_tracker.seconds_count = runtime;
    }
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
