use futures::Future;
use hyper::{Body, Request, Response, StatusCode};
use tokio_fs::file::File;
use tokio_io::io::read_to_end;

type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = std::io::Error> + Send>;

pub fn handle_request(req: Request<Body>) -> ResponseFuture {
    // FIXME(JTG): Add appropriate Content-Type headers
    if req.uri().path() == "/png" {
        serve_file("/tmp/chart.png")
    } else {
        serve_file("public/index.html")
    }
}

fn serve_file(file: &str) -> ResponseFuture {
    let filename = file.to_string();
    Box::new(
        File::open(filename)
            .and_then(|file| {
                let buf: Vec<u8> = Vec::new();
                read_to_end(file, buf)
                    .and_then(|item| Ok(Response::new(item.1.into())))
                    .or_else(|_| {
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::empty())
                            .unwrap())
                    })
            })
            .or_else(|_| {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap())
            }),
    )
}
