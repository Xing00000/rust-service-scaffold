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
