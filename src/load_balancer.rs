use std::str::FromStr;

use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Request, Response, Uri,
    body::{Bytes, Incoming},
};
use hyper_util::{
    client::legacy::{Client, Error, connect::HttpConnector},
    rt::TokioExecutor,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    balancing_algorithms::{BalancingAlgorithm, LeastConnectionsAlgorithm, RoundRobinAlgorithm},
    worker::Worker,
};

pub type ResponseBody = http_body_util::combinators::BoxBody<
    hyper::body::Bytes,
    Box<dyn std::error::Error + Send + Sync>,
>;

pub struct LoadBalancer {
    client: Client<HttpConnector, Incoming>,
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

    pub async fn handle_request(
        &self,
        mut req: Request<Incoming>,
    ) -> Result<hyper::Response<ResponseBody>, hyper_util::client::legacy::Error> {
        let worker = {
            self.balancing_algorithm
                .write()
                .await
                .choose(&self.worker_hosts)
        };
        let mut worker_uri = worker.host.clone();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        if req.uri().path().ends_with("change_algo") {
            if let Some(value) = self.change_algorithm(&req).await {
                return value;
            }
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).unwrap();

        // Clone the original request's headers and method
        let mut builder = Request::builder()
            .method(req.method().clone().to_owned())
            .uri(new_uri);
        builder
            .headers_mut()
            .unwrap()
            .extend(req.headers_mut().drain());

        let new_req = builder.body(req.into_body()).expect("request builder");

        let response = self.client.request(new_req).await;
        self.balancing_algorithm.write().await.release(worker);

        // Wrap the streaming response body in BoxBody
        response.map(|res| {
            let (parts, body) = res.into_parts();
            let boxed_body: ResponseBody = body
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                .boxed();
            Response::from_parts(parts, boxed_body)
        })
    }

    async fn change_algorithm(
        &self,
        req: &Request<Incoming>,
    ) -> Option<Result<Response<ResponseBody>, Error>> {
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

        let response_body: ResponseBody = ResponseBody::new(
            Full::new(Bytes::from("Balancing algorithm changed successfully"))
                .map_err(|infallible| match infallible {}),
        );

        return Some(Ok(Response::new(response_body)));
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
