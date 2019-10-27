# Twitter Streams

This is an example application demonstrating access to the Twitter Streaming API.

## Usage

### Build Dependencies

This crate uses async/await syntax, so a nightly compiler is necessary.  The beta channel does not
work at the moment due to some pinned dependencies, but it is expected the ecosystem will stabilize
quickly after 1.39.0 is released.

The charting functionality depends on a system library to draw text.  On an Ubuntu system, you can
run `sudo apt-get install libfontconfig libfontconfig1-dev libfreetype6-dev` (tested on Ubuntu
19.10).

### Setup

To run the application, first crate a `.env` file setting the following environment variables.
The required keys and secrets must be obtained from Twitter Developer Dashboard for your
application.

```
TWITTER_CONSUMER_KEY=<key>
TWITTER_CONSUMER_SECRET=<secret>
TWITTER_TOKEN_KEY=<key>
TWITTER_TOKEN_SECRET=<secret>
```

### Run the Server

The server can now be run with `cargo +nightly run --release`.

## Architecture

The application spawns two top-level tasks.  The "client task" makes a connection to the Twitter
Streaming API and runs sentiment analysis on the received tweets.  At 1 second intervals, this task
plots a line series graph of the accumulated data and saves it to a PNG on the local filesystem.

The other top-level task is the "server task".  It listens for incoming HTTP connections and serves
up either a static HTML page, or the PNG image.  The HTML file includes a meta tag to refresh the
page every 2 seconds.

The two top-level tasks only share a single piece of state (the PNG image) through the filesystem.
After writing to a new file, the client task renames the file into place so the server task never
observes a partially written file.

Note that for some reason the client sub-tasks are not being run concurrently.  If I introduce delay into
the sentiment analysis (via `thread::sleep`) then the next tweet is not processed until the
previous future completes.  With the current design I expect that up to 4 threads should be able to
sleep concurrently before stalling the processing of additional tweets.  If I add a similar sleep
delay to the server task (within a call to `tokio_executor::threadpool::blocking`), things behave
as expected and multiple HTTP clients are able to connect and sleep in parallel.  I have not yet
tracked down the source of this bug, however the existing processing is sufficiently fast to handle
the current stream of data sent by Twitter, even without this concurrency.

## Possible Enhancements

* It would be nice to move the legend outside of the chart area so that most recent data is not
  obscured, and to have thicker line weights in the plot.  Neither seems to be implemented upstream
  in the crate I picked.
* The page flickers with each reload, which isn't ideal.  It may be possible to load the image via
  AJAX and then draw the PNG via canvas or a data URL.  Or, the raw data could be sent for
  client-side rendering.  The current plotting library appears to support compilation to WASM and
  could possibly be used there.
* A WebSocket could be used to send data incrementally for client-side rendering.
* A debug assertion is hit within `miniz_oxide-0.3.4` when run in debug mode.  Check if this is
  reported upstream.

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
