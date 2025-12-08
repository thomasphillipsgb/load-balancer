use std::{net::SocketAddr, sync::Arc};

use http_body_util::combinators::BoxBody;
use hyper::server::conn::http1;
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use load_balancer::balancing_algorithms::LeastConnectionsAlgorithm;
use load_balancer::{LoadBalancer, Worker};
use tokio::{net::TcpListener, task};

#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
    ];

    let algo = Box::new(LeastConnectionsAlgorithm::new(&worker_hosts));

    let load_balancer =
        Arc::new(LoadBalancer::new(worker_hosts, algo).expect("failed to create load balancer"));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    println!("load balancer listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await.expect("failed to accept");
        println!("accepted connection from {}", stream.peer_addr().unwrap());
        let load_balancer = load_balancer.clone();

        task::spawn(async move {
            println!(
                "spawned task for connection from {}",
                stream.peer_addr().unwrap()
            );
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
    load_balancer: Arc<LoadBalancer>,
) -> Result<Response<load_balancer::ResponseBody>, hyper_util::client::legacy::Error> {
    load_balancer.handle_request(req).await
}
