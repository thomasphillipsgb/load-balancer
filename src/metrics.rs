use std::collections::HashMap;

use tokio::time::Instant;

use crate::{Worker, balancing_algorithms::AlgorithmType};

pub struct Metrics {
    average_response_time_for_algorithm: HashMap<AlgorithmType, u128>,
    request_count: u128,
    previous_switch_timestamp: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            average_response_time_for_algorithm: HashMap::new(),
            request_count: 0,
            previous_switch_timestamp: Instant::now(),
        }
    }

    pub fn record_response_time(&mut self, algorithm_type: AlgorithmType, time_ms: u128) {
        let entry = self
            .average_response_time_for_algorithm
            .entry(algorithm_type)
            .or_insert(0);
        *entry = ((*entry * self.request_count) + time_ms) / (self.request_count + 1);
        self.request_count += 1;
    }

    pub fn get_average_response_time_ms(&self, algorithm_type: AlgorithmType) -> u128 {
        *self
            .average_response_time_for_algorithm
            .get(&algorithm_type)
            .unwrap_or(&0)
    }

    pub fn reset(&mut self, algorithm_type: AlgorithmType) {
        self.average_response_time_for_algorithm
            .remove(&algorithm_type);
        self.request_count = 0;
        self.previous_switch_timestamp = Instant::now();
    }

    pub fn should_switch(&self) -> bool {
        self.previous_switch_timestamp.elapsed().as_secs() >= 10
    }
}
