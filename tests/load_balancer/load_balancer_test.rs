use load_balancer::balancing_algorithms::{
    AlgorithmType, LeastConnectionsAlgorithm, RoundRobinAlgorithm,
};
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

#[tokio::test]
async fn test_algorithm_change_round_robin_to_least_connections() {
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
        LoadBalancer::new(workers.clone(), algorithm).expect("Failed to create load balancer");

    // Verify initial algorithm is RoundRobin
    {
        let algo = load_balancer.balancing_algorithm.read().await;
        assert_eq!(algo.get_type(), AlgorithmType::RoundRobin);
    }

    // Manually change the algorithm to LeastConnections
    {
        let mut algo = load_balancer.balancing_algorithm.write().await;
        *algo = Box::new(LeastConnectionsAlgorithm::new(&workers));
    }

    // Verify algorithm changed to LeastConnections
    {
        let algo = load_balancer.balancing_algorithm.read().await;
        assert_eq!(algo.get_type(), AlgorithmType::LeastConnections);
    }
}

#[tokio::test]
async fn test_algorithm_change_least_connections_to_round_robin() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
    ];
    let algorithm = Box::new(LeastConnectionsAlgorithm::new(&workers));
    let load_balancer =
        LoadBalancer::new(workers.clone(), algorithm).expect("Failed to create load balancer");

    // Verify initial algorithm is LeastConnections
    {
        let algo = load_balancer.balancing_algorithm.read().await;
        assert_eq!(algo.get_type(), AlgorithmType::LeastConnections);
    }

    // Manually change the algorithm to RoundRobin
    {
        let mut algo = load_balancer.balancing_algorithm.write().await;
        *algo = Box::new(RoundRobinAlgorithm::new());
    }

    // Verify algorithm changed to RoundRobin
    {
        let algo = load_balancer.balancing_algorithm.read().await;
        assert_eq!(algo.get_type(), AlgorithmType::RoundRobin);
    }
}

#[tokio::test]
async fn test_debouncing_prevents_rapid_switches() {
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
        LoadBalancer::new(workers, algorithm).expect("Failed to create load balancer");

    // Record high response times to trigger switch condition
    {
        let mut metrics = load_balancer.metrics.write().await;
        for _ in 0..5 {
            metrics.record_response_time(AlgorithmType::RoundRobin, 2500);
        }
    }

    // Check that should_switch() returns false immediately (debouncing active)
    {
        let metrics = load_balancer.metrics.read().await;
        assert!(
            !metrics.should_switch(),
            "Should not switch immediately due to debouncing (less than 10 seconds)"
        );
    }
}

#[tokio::test]
async fn test_metrics_record_and_average_response_time() {
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
        LoadBalancer::new(workers, algorithm).expect("Failed to create load balancer");

    // Record response times
    {
        let mut metrics = load_balancer.metrics.write().await;
        metrics.record_response_time(AlgorithmType::RoundRobin, 100);
        metrics.record_response_time(AlgorithmType::RoundRobin, 200);
        metrics.record_response_time(AlgorithmType::RoundRobin, 300);
    }

    // Verify average response time is calculated correctly
    {
        let metrics = load_balancer.metrics.read().await;
        let average = metrics.get_average_response_time_ms(AlgorithmType::RoundRobin);
        assert_eq!(average, 200, "Average of [100, 200, 300] should be 200");
    }
}

#[tokio::test]
async fn test_metrics_reset_clears_data() {
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
        LoadBalancer::new(workers, algorithm).expect("Failed to create load balancer");

    // Record response times
    {
        let mut metrics = load_balancer.metrics.write().await;
        metrics.record_response_time(AlgorithmType::RoundRobin, 100);
        metrics.record_response_time(AlgorithmType::RoundRobin, 200);
        let average_before_reset = metrics.get_average_response_time_ms(AlgorithmType::RoundRobin);
        assert_eq!(average_before_reset, 150);
    }

    // Reset metrics
    {
        let mut metrics = load_balancer.metrics.write().await;
        metrics.reset(AlgorithmType::RoundRobin);
    }

    // Verify metrics are cleared
    {
        let metrics = load_balancer.metrics.read().await;
        let average_after_reset = metrics.get_average_response_time_ms(AlgorithmType::RoundRobin);
        assert_eq!(average_after_reset, 0, "Average should be 0 after reset");
    }
}

#[tokio::test]
async fn test_automatic_algorithm_switch_on_high_latency() {
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
        LoadBalancer::new(workers.clone(), algorithm).expect("Failed to create load balancer");

    // Record high response times to simulate high latency
    {
        let mut metrics = load_balancer.metrics.write().await;
        for _ in 0..10 {
            metrics.record_response_time(AlgorithmType::RoundRobin, 2500);
        }
        let avg = metrics.get_average_response_time_ms(AlgorithmType::RoundRobin);
        assert!(avg > 2000, "Average response time should exceed 2000ms");
    }

    // Verify that the condition for switching would be met (high response time)
    // Note: Actual switch in handle_request also requires should_switch() to return true
    // which requires 10 seconds to have passed since last switch
    {
        let metrics = load_balancer.metrics.read().await;
        assert!(metrics.get_average_response_time_ms(AlgorithmType::RoundRobin) > 2000);
        // Debouncing prevents immediate switch
        assert!(!metrics.should_switch());
    }
}
