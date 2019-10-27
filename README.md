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

**TODO**: Run in a clean install to see if any other system deps are needed.

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

**TODO**

## Possible Enhancements

* It would be nice to move the legend outside of the chart area so that most recent data is not
  obscured, and to have thicker line weights in the plot.  Neither seems to be implemented upstream
  in the crate I picked.
* The page flickers with each reload, which isn't ideal.  It may be possible to load the image via
  AJAX and then draw the PNG via canvas or a data URL.  Or, the raw data could be sent for
  client-side rendering.  The current plotting library appears to support compilation to WASM and
  could possibly be used there.
* A WebSocket could be used to send data incrementally for client-side rendering.
* A debug assertion is hit within `miniz_oxide-0.3.4` when run in debug mode.  See if this is
  reported upstream.

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
