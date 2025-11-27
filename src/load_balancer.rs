use std::{str::FromStr, sync::Arc};

use hyper::{Request, Uri, body::Incoming};
use hyper_util::{
    client::legacy::{Client, ResponseFuture, connect::HttpConnector},
    rt::TokioExecutor,
};
use tokio::sync::RwLock;

use crate::{balancing_algorithms::BalancingAlgorithm, worker::Worker};

pub struct LoadBalancer {
    client: Client<HttpConnector, Incoming>,
    worker_hosts: Vec<Worker>,
    current_worker: usize,
    balancing_algorithm: Box<dyn BalancingAlgorithm>,
}

impl LoadBalancer {
    pub fn new(
        worker_hosts: Vec<String>,
        balancing_algorithm: Box<dyn BalancingAlgorithm>,
    ) -> Result<Self, String> {
        if worker_hosts.is_empty() {
            return Err("No worker hosts provided".into());
        }
        let worker_hosts = worker_hosts
            .iter()
            .map(|host| Worker { host: host.clone() })
            .collect();

        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        Ok(LoadBalancer {
            client,
            worker_hosts,
            current_worker: 0,
            balancing_algorithm,
        })
    }

    pub fn forward_request(&mut self, req: Request<Incoming>) -> ResponseFuture {
        let worker_uri = self.get_worker().to_owned();
        let mut worker_uri = worker_uri.host.clone();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).unwrap();

        // Extract the headers from the original request
        let headers = req.headers().clone();

        // Clone the original request's headers and method
        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .body(req.into_body())
            .expect("request builder");

        // Copy headers from the original request
        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        self.client.request(new_req)
    }

    fn get_worker(&mut self) -> &Worker {
        // Use a round-robin strategy to select a worker
        let worker = self.worker_hosts.get(self.current_worker).unwrap();
        println!("hit worker {}", self.current_worker);
        self.current_worker = (self.current_worker + 1) % self.worker_hosts.len();
        worker
    }
}
