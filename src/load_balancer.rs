use std::{str::FromStr, sync::Arc};

use hyper::{Request, Uri, body::Incoming};
use hyper_util::{
    client::legacy::{Client, ResponseFuture, connect::HttpConnector},
    rt::TokioExecutor,
};

use crate::{balancing_algorithms::BalancingAlgorithm, worker::Worker};

pub struct LoadBalancer {
    client: Client<HttpConnector, Incoming>,
    worker_hosts: Vec<Worker>,
    balancing_algorithm: Box<dyn BalancingAlgorithm>,
}

impl LoadBalancer {
    pub fn new(
        worker_hosts: Vec<Worker>,
        balancing_algorithm: Box<dyn BalancingAlgorithm>,
    ) -> Result<Self, String> {
        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        Ok(LoadBalancer {
            client,
            worker_hosts,
            balancing_algorithm,
        })
    }

    pub async fn forward_request(
        &mut self,
        req: Request<Incoming>,
    ) -> Result<hyper::Response<Incoming>, hyper_util::client::legacy::Error> {
        let worker = self.balancing_algorithm.choose(&self.worker_hosts).unwrap();
        let mut worker_uri = worker.host.clone();

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

        let response = self.client.request(new_req).await;
        self.balancing_algorithm.release(worker);
        response
    }
}
