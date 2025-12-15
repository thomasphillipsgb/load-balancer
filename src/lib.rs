pub mod balancing_algorithms;
mod env;
mod load_balancer;

pub use load_balancer::{LoadBalancer, ResponseBody};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worker {
    pub host: String,
}
