pub async fn worker_service(
    req: Request<Incoming>,
    port: u16,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let message = format!(
        "worker on port {} received {} {}",
        port,
        req.method(),
        req.uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
    );

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from("Health Status - OK")))
            .expect("response builder")),
        (&Method::GET, "/work") => {
            tokio::time::sleep(Duration::from_millis(10)).await;

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("Work complete!")))
                .expect("response builder"))
        }
        _ => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from(message)))
            .expect("response builder")),
    }
}
