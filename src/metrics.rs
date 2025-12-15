use std::collections::HashMap;

use crate::{Worker, balancing_algorithms::AlgorithmType};

pub struct Metrics {
    pub average_response_time_for_algorithm: HashMap<AlgorithmType, f32>,
    request_count: f32,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            average_response_time_for_algorithm: HashMap::new(),
            request_count: 0f32,
        }
    }

    pub fn record_response_time(&mut self, algorithm_type: AlgorithmType, time: f32) {
        let entry = self
            .average_response_time_for_algorithm
            .entry(algorithm_type)
            .or_insert(0f32);
        *entry = ((*entry * self.request_count) + time) / (self.request_count + 1f32);
        self.request_count += 1f32;
    }

    pub fn get_average_response_time(&self, algorithm_type: AlgorithmType) -> Option<f32> {
        self.average_response_time_for_algorithm
            .get(&algorithm_type)
            .map(|&time| time)
    }

    pub fn reset(&mut self, algorithm_type: AlgorithmType) {
        self.average_response_time_for_algorithm
            .remove(&algorithm_type);
        self.request_count = 0f32;
    }
}
