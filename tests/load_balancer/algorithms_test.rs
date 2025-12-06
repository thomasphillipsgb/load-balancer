use load_balancer::Worker;
use load_balancer::balancing_algorithms::{
    BalancingAlgorithm, LeastConnections, RoundRobinAlgorithm,
};

#[test]
fn test_round_robin_algorithm_selection() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
        Worker {
            host: "http://localhost:3002".to_string(),
        },
    ];
    let mut algorithm = RoundRobinAlgorithm::new();

    // Test that round robin cycles through workers
    let first_worker = algorithm.choose(&workers);
    assert_eq!(first_worker.host, "http://localhost:3000");

    let second_worker = algorithm.choose(&workers);
    assert_eq!(second_worker.host, "http://localhost:3001");

    let third_worker = algorithm.choose(&workers);
    assert_eq!(third_worker.host, "http://localhost:3002");

    // Should wrap around
    let fourth_worker = algorithm.choose(&workers);
    assert_eq!(fourth_worker.host, "http://localhost:3000");
}

#[test]
fn test_least_connections_algorithm_selection() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
    ];
    let mut algorithm = LeastConnections::new(&workers);

    // Initially should choose first worker (they're equal at 0 connections)
    let first_choice = algorithm.choose(&workers);

    // Don't release the first worker, so second choice should be the other one
    let second_choice = algorithm.choose(&workers);

    // Should prefer the worker with fewer connections (the second one)
    assert_ne!(first_choice.host, second_choice.host);
}

#[test]
fn test_least_connections_release_functionality() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
    ];
    let mut algorithm = LeastConnections::new(&workers);

    // Choose a worker
    let chosen_worker = algorithm.choose(&workers);

    // Release it - should not panic
    algorithm.release(chosen_worker);
}

#[test]
fn test_round_robin_with_single_worker() {
    let workers = vec![Worker {
        host: "http://localhost:3000".to_string(),
    }];
    let mut algorithm = RoundRobinAlgorithm::new();

    // Should always return the same worker
    for _ in 0..5 {
        let chosen = algorithm.choose(&workers);
        assert_eq!(chosen.host, "http://localhost:3000");
    }
}

#[test]
fn test_least_connections_initialization() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
    ];
    let _algorithm = LeastConnections::new(&workers);

    // Should create successfully without panicking
    assert!(true);
}

#[test]
fn test_worker_creation() {
    let worker = Worker {
        host: "http://localhost:3000".to_string(),
    };

    assert_eq!(worker.host, "http://localhost:3000");
}

#[test]
fn test_worker_clone() {
    let worker1 = Worker {
        host: "http://localhost:3000".to_string(),
    };
    let worker2 = worker1.clone();

    assert_eq!(worker1.host, worker2.host);
    assert_eq!(worker1, worker2);
}

#[test]
fn test_round_robin_algorithm_consistency() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
    ];
    let mut algorithm = RoundRobinAlgorithm::new();

    // Test multiple cycles
    let mut selections = vec![];
    for _ in 0..6 {
        // 3 full cycles
        let worker = algorithm.choose(&workers);
        selections.push(worker.host.clone());
    }

    // Should follow pattern: 3000, 3001, 3000, 3001, 3000, 3001
    assert_eq!(
        selections,
        vec![
            "http://localhost:3000",
            "http://localhost:3001",
            "http://localhost:3000",
            "http://localhost:3001",
            "http://localhost:3000",
            "http://localhost:3001"
        ]
    );
}

#[test]
fn test_least_connections_prefers_less_busy_worker() {
    let workers = vec![
        Worker {
            host: "http://localhost:3000".to_string(),
        },
        Worker {
            host: "http://localhost:3001".to_string(),
        },
        Worker {
            host: "http://localhost:3002".to_string(),
        },
    ];
    let mut algorithm = LeastConnections::new(&workers);

    // Choose first worker and don't release it
    let worker1 = algorithm.choose(&workers);

    // Choose second worker and don't release it
    let worker2 = algorithm.choose(&workers);
    assert_ne!(worker1.host, worker2.host);

    // Third choice should be the third worker (least connections)
    let worker3 = algorithm.choose(&workers);
    assert_ne!(worker3.host, worker1.host);
    assert_ne!(worker3.host, worker2.host);
}
