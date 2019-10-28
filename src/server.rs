use hyper::{Body, Request, Response, Result, StatusCode};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Serve the PNG on a single path, and the index on all other routes
pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>> {
    // FIXME(JTG): Add appropriate Content-Type headers
    if req.uri().path() == "/png" {
        Ok(serve_file("/tmp/chart.png").await)
    } else {
        Ok(serve_file("public/index.html").await)
    }
}

/// Load file asynchronously and return a Response
///
/// If there is an error loading the file a status 500 response will be generated
async fn serve_file(file: &str) -> Response<Body> {
    if let Ok(mut file) = File::open(file).await {
        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).await.is_ok() {
            return Response::new(buf.into());
        }
    }
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap()
}
