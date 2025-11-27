use load_balancer::LoadBalancer;

#[tokio::test]
async fn should_hit_monitor() {
    let load_balancer = LoadBalancer::new(vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
    ]);
}
