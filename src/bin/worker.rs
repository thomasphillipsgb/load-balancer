use std::{convert::Infallible, env, net::SocketAddr, time::Duration};

use http_body_util::Full;
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    service::service_fn,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use load_balancer::worker_service;
use tokio::{net::TcpListener, task};

#[tokio::main]
async fn main() {
    let port = env::args()
        .nth(1)
        .and_then(|port| port.parse().ok())
        .or_else(|| env::var("PORT").ok().and_then(|port| port.parse().ok()))
        .unwrap_or(3000);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("worker listening on http://{}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind worker port");

    loop {
        let (stream, _) = listener.accept().await.expect("failed to accept");

        task::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| worker_service(req, port));
            let builder = Builder::new(TokioExecutor::new());

            if let Err(err) = builder.serve_connection(io, service).await {
                eprintln!("worker connection error: {err}");
            }
        });
    }
}
