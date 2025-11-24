use std::{net::SocketAddr, str::FromStr, sync::Arc};

use hyper::server::conn::http1;
use hyper::{Request, Response, Uri, body::Incoming, service::service_fn};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::{Client, Error as ClientError, ResponseFuture};
use hyper_util::rt::{TokioExecutor, TokioIo};
use load_balancer::LoadBalancer;
use tokio::sync::RwLock;
use tokio::{net::TcpListener, task};

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
    ];

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(worker_hosts).expect("failed to create load balancer"),
    ));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    println!("load balancer listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await.expect("failed to accept");
        let load_balancer = load_balancer.clone();

        task::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| handle(req, load_balancer.clone()));

            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("error: {}", e);
            }
        });
    }
}

async fn handle(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Incoming>, ClientError> {
    load_balancer.write().await.forward_request(req).await
}
