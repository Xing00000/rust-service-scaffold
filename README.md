# rust-service-scaffold
rust-service-scaffold

## Telemetry Configuration (OpenTelemetry)

This service is integrated with OpenTelemetry to provide distributed tracing and metrics.

### Overview

-   **Distributed Tracing**: Captures request flows and performance data, exportable to Jaeger.
-   **Metrics**: Collects key performance indicators (e.g., HTTP request counts, latency), exportable to Prometheus.

### Configuration via Environment Variables

The telemetry system can be configured using the following environment variables:

-   `RUST_LOG`: Controls log level (e.g., `info,axum_logging_service=debug`).
-   `OTEL_SERVICE_NAME`: Identifies your service in telemetry backends.
    -   Example: `OTEL_SERVICE_NAME="my-awesome-app"`
    -   Default: `axum-service` (as per `telemetry.rs`)

#### Jaeger Exporter (Traces)

The service is configured to export traces to a Jaeger agent.

-   `OTEL_EXPORTER_JAEGER_AGENT_HOST`: The hostname for the Jaeger agent.
    -   Default: `localhost` (as per OpenTelemetry SDK defaults if not overridden by Jaeger specific env vars)
    -   Example: `OTEL_EXPORTER_JAEGER_AGENT_HOST="127.0.0.1"` or `"jaeger"` if running in Docker.
-   `OTEL_EXPORTER_JAEGER_AGENT_PORT`: The port for the Jaeger agent's compact thrift protocol.
    -   Default: `6831` (as per OpenTelemetry SDK defaults if not overridden by Jaeger specific env vars)
    -   Example: `OTEL_EXPORTER_JAEGER_AGENT_PORT="6831"`

*Note: For other Jaeger setups, such as exporting directly to a Jaeger collector via HTTP or gRPC, you might use variables like `OTEL_EXPORTER_JAEGER_ENDPOINT`. The current implementation uses the agent pipeline which is configured by the above variables or their SDK defaults.*

#### Prometheus Exporter (Metrics)

Metrics are exposed via a `/metrics` endpoint on the application's server address.

-   Example: If the service runs on `http://localhost:3000`, metrics will be available at `http://localhost:3000/metrics`.
-   Prometheus can be configured to scrape this endpoint.
-   The `OTEL_SERVICE_NAME` is also used as a resource attribute for metrics, helping to identify them in Prometheus.

### How to Run with Telemetry Backends (Local Example)

You can use Docker Compose to easily run Jaeger and Prometheus locally.

1.  **Create `docker-compose.yml`**:
    ```yaml
    version: '3.8'
    services:
      jaeger:
        image: jaegertracing/all-in-one:latest # Includes agent, collector, query, and UI
        ports:
          - "6831:6831/udp" # Jaeger agent UDP port for compact thrift protocol
          - "6832:6832/udp" # Jaeger agent UDP port for binary thrift protocol (if needed by client)
          - "5778:5778/tcp"  # Jaeger agent HTTP port for config (rarely used by clients)
          - "16686:16686"  # Jaeger UI
          - "14268:14268"  # Jaeger collector HTTP port for traces (e.g., for direct HTTP exporter)
          # - "4317:4317"    # OTLP gRPC port (if Jaeger collector is configured for OTLP)
          # - "4318:4318"    # OTLP HTTP port (if Jaeger collector is configured for OTLP)
        environment:
          - COLLECTOR_ZIPKIN_HOST_PORT=:9411 # For Zipkin compatibility if needed
          # For newer Jaeger versions, OTLP is often enabled by default or via specific OTLP env vars.
          # Example: JAEGER_OTLP_GRPC_HOST_PORT=:4317 or similar depending on Jaeger version.
          # The opentelemetry-jaeger crate by default uses agent, not OTLP to Jaeger.

      prometheus:
        image: prom/prometheus:latest
        volumes:
          - ./prometheus.yml:/etc/prometheus/prometheus.yml # Mount prometheus config
        ports:
          - "9090:9090"
        command:
          - '--config.file=/etc/prometheus/prometheus.yml'
    ```

2.  **Create `prometheus.yml`** (in the same directory as `docker-compose.yml`):
    ```yaml
    global:
      scrape_interval: 15s # How frequently to scrape targets

    scrape_configs:
      - job_name: 'axum-service-telemetry'
        # Adjust the target based on where your Axum service is accessible from the Prometheus container.
        # If your Axum service is running on your host machine (not in Docker):
        # For Docker on Linux, 'localhost' might work if network_mode=host, or use host IP.
        # For Docker Desktop (Windows/macOS), use 'host.docker.internal'.
        static_configs:
          - targets: ['host.docker.internal:3000']
        # If your Axum service is also running in a Docker container on the same Docker network:
        # static_configs:
        #   - targets: ['your_axum_app_container_name:3000']
    ```

3.  **Start backends**:
    ```bash
    docker-compose up -d
    ```

4.  **Run your Axum service** with appropriate environment variables:
    ```bash
    export OTEL_SERVICE_NAME="my-axum-app"
    # If Jaeger is running via Docker Compose as above, and your app is on the host,
    # the default Jaeger agent host (localhost) and port (6831) should work.
    # export OTEL_EXPORTER_JAEGER_AGENT_HOST="localhost"
    # export OTEL_EXPORTER_JAEGER_AGENT_PORT="6831"
    cargo run
    ```
    (Ensure your application is listening on `0.0.0.0:3000` or similar to be accessible from `host.docker.internal` or other containers, not just `127.0.0.1:3000` which would only be accessible from the host itself). Your current `main.rs` uses `127.0.0.1:3000`, this might need to be changed to `0.0.0.0:3000` for the Dockerized Prometheus to scrape it.

5.  **Access Telemetry Data**:
    -   **Jaeger UI** (Traces): Open `http://localhost:16686` in your browser. Select your service (`my-axum-app` or whatever `OTEL_SERVICE_NAME` you set) and find traces.
    -   **Prometheus UI** (Metrics): Open `http://localhost:9090`. You can query metrics like `http_server_active_requests`, `http_server_duration_seconds_count`, or `http_server_duration_seconds_bucket`. The exact metric names are defined by the OpenTelemetry HTTP semantic conventions. Check the `/metrics` endpoint of your service (e.g., `http://localhost:3000/metrics`) for available metrics.

## Rate Limiting (Tower Governor)

This service uses `tower_governor` for rate limiting to protect against excessive requests.

### Configuration

The rate limiter is configured in `src/app.rs`. By default, it is set to allow a burst size of 2 requests per second.

```rust
// Example configuration in src/app.rs
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
// ...
    .layer(
        GovernorLayer::new(&Arc::new(
            GovernorConfigBuilder::default()
                .burst_size(2) // Allow 2 requests per second
                .finish()
                .unwrap(),
        ))
    );
```

You can adjust the `burst_size` and other parameters (like `per_second`, `per_minute`, etc.) in `GovernorConfigBuilder` as needed. Refer to the `tower_governor` documentation for more advanced configurations.

### Integration with Redis for Multi-Replica Deployments

When deploying multiple instances of the service, a distributed rate limiter is necessary to ensure consistent behavior. `tower_governor` supports using Redis as a backend for this purpose.

To integrate with Redis:

1.  **Add Redis dependencies**:
    You'll need to add `tower_governor` with the `redis` feature and a Redis client like `r2d2_redis` or `redis` (async).
    ```toml
    # In Cargo.toml
    tower_governor = { version = "0.4.1", features = ["redis"] }
    r2d2 = "0.8"
    r2d2_redis = "0.14" # Or an async redis client
    # OR if using async redis directly:
    # redis = { version = "0.23", features = ["tokio-comp"] }
    ```

2.  **Configure Governor with RedisStateStore**:
    Modify `src/app.rs` to use `GovernorConfigBuilder::use_state_store` with a Redis connection pool.

    ```rust
    // Example (conceptual) for src/app.rs using r2d2_redis:
    use tower_governor::{
        governor::GovernorConfigBuilder,
        key_extractor::SmartIpKeyExtractor, // To rate limit by IP
        GovernorLayer,
        RedisStateStore, // If using the "redis" feature
    };
    use r2d2_redis::{r2d2, RedisConnectionManager};
    use std::sync::Arc;
    // ...

    // Inside your Application::build or similar setup function:

    // 1. Setup Redis connection pool
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let manager = RedisConnectionManager::new(redis_url).expect("Failed to create Redis manager");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create Redis pool");

    // 2. Create the Redis state store
    let state_store = RedisStateStore::new(pool);

    // 3. Build the governor config using the Redis store
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor) // Rate limit based on IP
            .burst_size(10) // Example: 10 requests
            .period(std::time::Duration::from_secs(60)) // Per minute
            .use_state_store(state_store)
            .finish()
            .unwrap(),
    );

    // 4. Add the GovernorLayer
    // ...
    // .layer(GovernorLayer::new(&governor_conf))
    // ...
    ```
    Ensure your Redis server is running and accessible by the application. You'll need to set the `REDIS_URL` environment variable.

    **Note**: The above Redis integration example is conceptual. You'll need to adapt it to your specific error handling, configuration management, and choice of synchronous/asynchronous Redis client. For an asynchronous setup (which is generally preferred with Axum/Tokio), you would use an async Redis client and an async-compatible state store if `tower_governor` provides one directly or if you build one. The current `RedisStateStore` in `tower_governor` might expect a blocking client, so careful integration is needed. If `tower_governor`'s `RedisStateStore` is blocking, running it in `tokio::spawn_blocking` might be necessary for each state access, or use a dedicated thread pool for Redis operations. Always check the `tower_governor` documentation for the most up-to-date practices for Redis integration.

### Custom Instrumentation

-   **Custom Spans**: You can add custom child spans within your application logic using `tracing` macros like `tracing::info_span!`, `tracing::debug_span!`, etc. These will be automatically correlated with the parent request span due to the `tracing-opentelemetry` layer.
    ```rust
    // Example from the service's handler function:
    let _custom_work_span_guard = tracing::info_span!(
        "custom_work_in_handler",
        service_operation = "generate_greeting", // custom attributes
        request_id = %request_id // another custom attribute
    ).entered(); // Enters the span; it becomes current until the guard is dropped.

    tracing::info!("This log message is part of the custom_work_in_handler span.");
    ```
-   **Custom Metrics**: While not extensively demonstrated in the current template, custom metrics (e.g., business-specific counters, gauges, histograms) can be created using the OpenTelemetry Metrics API (`opentelemetry::metrics`). You would typically obtain a `Meter` from the global `MeterProvider` (which was initialized in `main.rs`) and use it to create and record metric instruments. Refer to the `opentelemetry` crate documentation for details on creating custom metrics.
find . -path ./target -prune -o -type f -name "*.rs" -print | while read file; do
  echo "=== $file ===" >> all_code.txt
  cat "$file" >> all_code.txt
  echo -e "\n" >> all_code.txt
done