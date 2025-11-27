use crate::worker::Worker;

pub trait BalancingAlgorithm: Send + Sync {
    fn choose<'a>(&mut self, workers: &'a Vec<Worker>) -> Option<&'a Worker>;
}

#[derive(Debug, Clone, Copy)]
pub struct RoundRobinAlgorithm {
    current_index: usize,
}

impl RoundRobinAlgorithm {
    pub fn new() -> Self {
        RoundRobinAlgorithm { current_index: 0 }
    }
}

impl BalancingAlgorithm for RoundRobinAlgorithm {
    fn choose<'a>(&mut self, workers: &'a Vec<Worker>) -> Option<&'a Worker> {
        if workers.is_empty() {
            return None;
        }
        let worker = &workers[self.current_index % workers.len()];
        self.current_index = (self.current_index + 1) % workers.len();
        Some(worker)
    }
}
