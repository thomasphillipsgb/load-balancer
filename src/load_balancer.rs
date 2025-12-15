use std::str::FromStr;

use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Request, Response, Uri,
    body::{Bytes, Incoming},
};
use hyper_util::{
    client::legacy::{Client, Error as ClientError, connect::HttpConnector},
    rt::TokioExecutor,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    Worker,
    balancing_algorithms::{
        AlgorithmType, BalancingAlgorithm, LeastConnectionsAlgorithm, RoundRobinAlgorithm,
    },
    metrics::Metrics,
};

pub type ResponseBody = http_body_util::combinators::BoxBody<
    hyper::body::Bytes,
    Box<dyn std::error::Error + Send + Sync>,
>;

pub struct LoadBalancer {
    client: Client<HttpConnector, Incoming>,
    worker_hosts: Vec<Worker>,
    balancing_algorithm: RwLock<Box<dyn BalancingAlgorithm>>,
    metrics: RwLock<Metrics>,
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
            metrics: RwLock::new(Metrics::new()),
        })
    }

    pub async fn handle_request(
        &self,
        mut req: Request<Incoming>,
    ) -> Result<hyper::Response<ResponseBody>, hyper_util::client::legacy::Error> {
        if req.uri().path().ends_with("change_algorithm") {
            return self.change_algorithm(&req).await;
        }

        let (worker, algo_type) = {
            let mut algo = self.balancing_algorithm.write().await;
            let algo_type = algo.get_type();

            let mut metrics = self.metrics.write().await;

            if metrics.get_average_response_time(algo_type).unwrap_or(0.0) > 2.0 {
                if algo.get_type() == AlgorithmType::LeastConnections {
                    metrics.reset(algo_type);
                    *algo = Box::new(RoundRobinAlgorithm::new());
                    println!("Switching to RoundRobinAlgorithm");
                } else {
                    // Switch to LeastConnectionsAlgorithm
                    metrics.reset(algo_type);
                    *algo = Box::new(LeastConnectionsAlgorithm::new(&self.worker_hosts));
                    println!("Switching to LeastConnectionsAlgorithm");
                }
            }
            (algo.choose(&self.worker_hosts), algo_type)
        };

        let mut worker_uri = worker.host.clone();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
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

        let before_time = std::time::Instant::now();

        let response = self.client.request(new_req).await;

        let elapsed_time = before_time.elapsed().as_secs_f32();

        self.balancing_algorithm.write().await.release(worker);
        self.metrics
            .write()
            .await
            .record_response_time(algo_type, elapsed_time);

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
    ) -> Result<Response<ResponseBody>, ClientError> {
        let response_body = match req.uri().query() {
            Some(query) => match serde_urlencoded::from_str::<ChangeAlgoRequest>(query) {
                Ok(params) => {
                    let new_algo: Box<dyn BalancingAlgorithm> = match params.algo_type {
                        BalancingAlgorithmType::RoundRobin => Box::new(RoundRobinAlgorithm::new()),
                        BalancingAlgorithmType::LeastConnections => {
                            Box::new(LeastConnectionsAlgorithm::new(&self.worker_hosts))
                        }
                    };
                    let mut algo = self.balancing_algorithm.write().await;
                    *algo = new_algo;
                    ResponseBody::new(
                        "Algorithm Changed!"
                            .to_string()
                            .map_err(|infallible| match infallible {}),
                    )
                }
                Err(_) => ResponseBody::new(
                    "Invalid Algorithm Type"
                        .to_string()
                        .map_err(|infallible| match infallible {}),
                ),
            },
            None => ResponseBody::new(
                "No Query Attached"
                    .to_string()
                    .map_err(|infallible| match infallible {}),
            ),
        };

        Ok(Response::new(response_body))
    }
}

#[derive(Deserialize)]
struct ChangeAlgoRequest {
    algo_type: BalancingAlgorithmType,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum BalancingAlgorithmType {
    RoundRobin,
    LeastConnections,
}
