pub mod balancing_algorithms;
mod load_balancer;
mod metrics;

pub use load_balancer::{LoadBalancer, ResponseBody};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Worker {
    pub host: String,
}
