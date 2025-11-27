use std::collections::HashMap;

pub struct LeastConnections {
    connection_map: HashMap<String, i32>,
}

impl LeastConnections {
    pub fn new() -> Self {
        Self {
            connection_map: HashMap::new(),
        }
    }
    pub fn choose<'a>(&mut self, workers: &'a Vec<String>) -> Option<&'a String> {
        // ensure we track connections to workers
        // look at each worker, finding the one with least connection and return it
        todo!();
    }
}
