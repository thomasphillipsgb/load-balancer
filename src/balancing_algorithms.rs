use std::collections::HashMap;

use crate::worker::{self, Worker};

pub trait BalancingAlgorithm: Send + Sync {
    fn choose<'a>(&mut self, workers: &'a Vec<Worker>) -> Option<&'a Worker>;
    fn release(&mut self, worker: &Worker) {
        let _ = worker;
    }
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
        println!("Chosen worker: {}", worker.host);
        Some(worker)
    }
}

pub struct LeastConnections {
    connection_map: HashMap<String, i32>,
}

impl LeastConnections {
    pub fn new(workers: &Vec<Worker>) -> Self {
        let mut connection_map = HashMap::new();
        for worker in workers {
            connection_map.insert(worker.host.clone(), 0);
        }
        Self { connection_map }
    }
}

impl BalancingAlgorithm for LeastConnections {
    fn choose<'a>(&mut self, workers: &'a Vec<Worker>) -> Option<&'a Worker> {
        for worker in workers {
            self.connection_map.entry(worker.host.clone()).or_insert(0);
        }

        let chosen_one = workers
            .iter()
            .min_by_key(|worker| self.connection_map.get(&worker.host).unwrap_or(&0));

        if let Some(worker) = chosen_one {
            let counter = self.connection_map.entry(worker.host.clone()).or_insert(0);
            *counter += 1;

            println!(
                "Chosen worker: {}, current connections: {}",
                worker.host, *counter
            );
        }

        Some(chosen_one.unwrap())
    }

    fn release(&mut self, worker: &Worker) {
        if let Some(counter) = self.connection_map.get_mut(&worker.host) {
            if *counter > 0 {
                *counter -= 1;
            }
            println!(
                "Released worker: {}, current connections: {}",
                worker.host, *counter
            );
        }
    }
}
