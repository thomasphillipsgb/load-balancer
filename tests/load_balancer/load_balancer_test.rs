use load_balancer::balancing_algorithms::RoundRobinAlgorithm;
use load_balancer::{LoadBalancer, Worker};
use std::sync::Arc;

#[tokio::test]
async fn test_load_balancer_new_with_valid_workers() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
    ];
    let algorithm = Box::new(RoundRobinAlgorithm::new());

    let load_balancer = LoadBalancer::new(workers, algorithm);
    assert!(load_balancer.is_ok());
}

#[tokio::test]
async fn test_load_balancer_new_with_empty_workers() {
    let workers = vec![];
    let algorithm = Box::new(RoundRobinAlgorithm::new());

    let load_balancer = LoadBalancer::new(workers, algorithm);
    assert!(load_balancer.is_err());
}

#[tokio::test]
async fn test_load_balancer_thread_safety() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
    ];
    let algorithm = Box::new(RoundRobinAlgorithm::new());

    let load_balancer =
        Arc::new(LoadBalancer::new(workers, algorithm).expect("Failed to create load balancer"));

    // Test that LoadBalancer can be shared across async tasks
    let lb1 = load_balancer.clone();
    let lb2 = load_balancer.clone();

    let task1 = tokio::spawn(async move {
        // This test verifies the LoadBalancer can be shared across tasks
        let _ = &lb1; // Use the load balancer reference
    });

    let task2 = tokio::spawn(async move {
        let _ = &lb2; // Use the load balancer reference
    });

    // Both tasks should complete without issues
    let (result1, result2) = tokio::join!(task1, task2);
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}
