use std::str::FromStr;

use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Request, Response, Uri,
    body::{Bytes, Incoming},
};
use hyper_util::{
    client::legacy::{Client, connect::HttpConnector},
    rt::TokioExecutor,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    balancing_algorithms::{BalancingAlgorithm, LeastConnectionsAlgorithm, RoundRobinAlgorithm},
    worker::Worker,
};

pub struct LoadBalancer {
    client: Client<HttpConnector, BoxBody<Bytes, hyper::Error>>,
    worker_hosts: Vec<Worker>,
    balancing_algorithm: RwLock<Box<dyn BalancingAlgorithm>>,
}

impl LoadBalancer {
    pub fn new(
        worker_hosts: Vec<Worker>,
        balancing_algorithm: Box<dyn BalancingAlgorithm>,
    ) -> Result<Self, String> {
        if worker_hosts.is_empty() {
            return Err("Worker hosts list cannot be empty".to_string());
        }

        let connector = HttpConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);

        Ok(LoadBalancer {
            client,
            worker_hosts,
            balancing_algorithm: RwLock::new(balancing_algorithm),
        })
    }

    pub async fn forward_request(
        &self,
        req: Request<Incoming>,
    ) -> Result<
        hyper::Response<BoxBody<Bytes, hyper::Error>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let worker = {
            self.balancing_algorithm
                .write()
                .await
                .choose(&self.worker_hosts)
                .unwrap()
        };
        let mut worker_uri = worker.host.clone();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        if req.uri().path().ends_with("change_algo") {
            let query = req.uri().query().expect("No Query");
            let params: ChangeAlgoRequest =
                serde_urlencoded::from_str(query).expect("Query is invalid for request");

            let new_algo: Box<dyn BalancingAlgorithm> = match params.algo_type {
                BalancingAlgorithmType::RoundRobin => Box::new(RoundRobinAlgorithm::new()),
                BalancingAlgorithmType::LeastConnections => {
                    Box::new(LeastConnectionsAlgorithm::new(&self.worker_hosts))
                }
            };

            let mut algo = self.balancing_algorithm.write().await;
            *algo = new_algo;

            return Ok(Response::builder()
                .body(BoxBody::new(Empty::new().map_err(|never| match never {})))
                .unwrap());
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).unwrap();

        // Destructure the request to extract parts without cloning
        let (mut parts, body) = req.into_parts();
        parts.uri = new_uri;

        // Reconstruct the request with the streaming body wrapped in BoxBody
        let new_req = Request::from_parts(parts, BoxBody::new(body));

        let response = self
            .client
            .request(new_req)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        self.balancing_algorithm.write().await.release(worker);

        // Wrap the streaming response body in BoxBody
        let (parts, body) = response.into_parts();
        let boxed_body = BoxBody::new(body.map_err(|e| hyper::Error::from(e)));
        Ok(Response::from_parts(parts, boxed_body))
    }
}

#[derive(Deserialize)]
struct ChangeAlgoRequest {
    algo_type: BalancingAlgorithmType,
}

#[derive(Deserialize)]
enum BalancingAlgorithmType {
    RoundRobin,
    LeastConnections,
}
