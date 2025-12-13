use std::{convert::Infallible, time::Duration};

use http_body_util::Full;
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
};

pub async fn worker_service(
    req: Request<Incoming>,
    port: u16,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let message = format!(
        "worker on port {} received {} {}",
        port,
        req.method(),
        req.uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
    );

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from("Health Status - OK\r\n")))
            .expect("response builder")),
        (&Method::GET, "/heavy_work") => {
            tokio::time::sleep(Duration::from_secs(10)).await;

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("Heavy Work complete!\r\n")))
                .expect("response builder"))
        }
        (&Method::GET, "/work") => {
            tokio::time::sleep(Duration::from_secs(1)).await;

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("Work complete!\r\n")))
                .expect("response builder"))
        }
        _ => Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from(message)))
            .expect("response builder")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worker {
    pub host: String,
}
