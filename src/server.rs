use hyper::{Body, Request, Response, Result, StatusCode};
use tokio::io::AsyncReadExt;
use tokio_fs::File;

pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>> {
    // FIXME(JTG): Add appropriate Content-Type headers
    if req.uri().path() == "/png" {
        serve_file("/tmp/chart.png").await
    } else {
        serve_file("public/index.html").await
    }
}

async fn serve_file(file: &str) -> Result<Response<Body>> {
    if let Ok(mut file) = File::open(file).await {
        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).await.is_ok() {
            return Ok(Response::new(buf.into()));
        }
    }
    Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap())
}
