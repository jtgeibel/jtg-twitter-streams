# Twitter Streams

This is an example application demonstrating access to the Twitter Streaming API.

## Usage

### Build Dependencies

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

The server can now be run with `cargo run --release`.

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
