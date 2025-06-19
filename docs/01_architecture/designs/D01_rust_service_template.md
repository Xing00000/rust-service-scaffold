# **設計文檔 D01: Rust 服務模板架構 v1.1**

**文檔版本**: 1.1
**狀態**: 已批准 (Approved)
**作者**: AI Architect, Community Reviewer
**最後更新**: 2024-06-18
**相關 Ticket**: 服務模板架構設計

---

## **1. 願景與目標 (Vision & Goals)**

### 1.1. 問題陳述

在微服務架構中，如果沒有統一的標準，每個新服務的開發都會面臨重複的基礎設施工作，如日誌、監控、錯誤處理等。這不僅降低了開發效率，更導致了各服務實現不一、可觀測性參差不齊、維護成本高昂等問題。

### 1.2. 設計目標

本設計旨在創建一個標準化的 **Rust 服務模板**，為所有微服務提供一個堅實、一致的基礎。其核心目標是：

- **開發效率 (Developer Velocity)**: 開發者可以基於模板快速創建新服務，專注於業務邏輯而非基礎設施。
- **默認可觀測性 (Observability by Default)**: 所有服務從誕生之初就具備結構化日誌、分佈式追踪和核心指標的能力。
- **穩健性與一致性 (Robustness & Consistency)**: 提供統一的錯誤處理和 Panic 捕獲機制，確保服務行為一致且可預測。
- **生產級就緒 (Production Ready)**: 內置 CI/CD 流程、運維端點和安全考量，縮短從開發到部署的距離。

---

## **2. 核心原則 (Core Principles)**

- **約定優於配置 (Convention over Configuration)**: 提供一套合理的默認配置，讓開發者開箱即用。
- **結構化與可操作日誌 (Structured & Actionable Logs)**: 所有日誌輸出為 JSON 格式，並自動注入上下文信息（如 `trace_id`），便於機器解析和問題定位。
- **優雅降級 (Graceful Degradation)**: 當遙測後端（如 Jaeger）不可用時，服務應能正常運行，僅記錄警告而不應崩潰。
- **安全與韌性 (Security & Resilience)**: 內置可選的限流策略，並提供清晰的外部依賴處理模式。
- **性能感知 (Performance Aware)**: 採用非阻塞日誌和高性能庫，確保基礎設施開銷最小化。

---

## **3. 整體架構 (Overall Architecture)**

服務模板的核心圍繞 Axum 框架，並通過中間件（Layers）將橫切關注點無縫集成。

```mermaid
graph TD
    subgraph "外部請求 (Incoming Request)"
        A[HTTP/gRPC]
    end

    subgraph "Axum 服務"
        B(Router)
        L0(Rate Limiter Layer)
        L1(Panic Catcher Layer)
        L2(OpenTelemetry Layer)
        L3(Tracing Layer)
        H(API Handler)
    end

    subgraph "核心組件"
        C(Configuration)
        T(Telemetry Initializer)
        E(Error Handling)
        P(Panic Hook & Task Guard)
    end

    subgraph "外部系統"
        O1[STDOUT/Loki (Logs)]
        O2[Jaeger/OTLP Collector (Traces)]
        O3[Prometheus (Metrics)]
    end

    C --> T
    T --> L2
    T --> L3
    T --> P

    A --> B
    B -- R L0 -- R L1 -- R L2 -- R L3 --> H
    H -- "Result<_, AppError>" --> E
    E --> A

    L3 -- "Non-blocking" --> O1
    L2 -- "Export" --> O2
    L2 -- "Export" --> O3
```

- **配置 (Configuration)**: 統一管理所有組件的配置。
- **遙測初始化 (Telemetry Initializer)**: 在服務啟動時，根據配置初始化日誌、追踪、指標和 Panic Hook。
- **中間件棧 (Middleware Stack)**: 請求依次通過可選的限流、Panic 捕獲、OpenTelemetry 和 Tracing 中間件，自動注入上下文。
- **錯誤處理 (Error Handling)**: API Handler 返回統一的 `Result`，由 `IntoResponse` 實現將錯誤轉換為標準的 API 響應。
- **Panic 處理**: 通過全局 Hook 和安全的任務生成器（Task Guard）捕獲所有 Panic。

---

## **4. 組件詳細設計 (Component Design)**

### **4.1. 日誌: `tracing`**

- **核心庫**: `tracing`, `tracing-subscriber`, `tracing-appender`, `tracing-bunyan-formatter`
- **模塊**: `src/telemetry.rs`

#### 設計要點:

1.  **非阻塞日誌**: 在生產環境中，使用 `tracing-appender` 將日誌寫入一個非阻塞的滾動文件或 STDOUT，將 I/O 操作與請求處理線程解耦，避免在高負載下阻塞服務。
2.  **結構化日誌 (JSON)**: 生產環境默認使用 `tracing-bunyan-formatter` 輸出 JSON 格式日誌，便於 Loki 或 Elasticsearch 採集和索引。
3.  **開發環境友好**: 開發環境使用 `tracing_subscriber::fmt` 進行美化、著色的控制台輸出。
4.  **配置驅動**: `log.format` (`"json"`/`"text"`) 和 `log.output` (`"stdout"`/`"file"`) 均可通過配置驅動。
5.  **統一初始化**: `init_subscriber` 函數根據配置構建並設置全局 `tracing` 訂閱者。

#### 接口定義 (偽代碼):

```rust
// src/telemetry.rs
// 根據配置決定是輸出 JSON 還是人類可讀的日誌，以及是否使用非阻塞 Writer。
pub fn get_subscriber(config: &LogConfig) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::new(&config.level);
    let formatter_layer = ...; // 根據 config.format 選擇 Bunyan 或 Fmt

    // 根據 config.output 選擇 non_blocking_writer 或 stdout
    let writer = ...;

    Registry::default().with(env_filter).with(formatter_layer).with_writer(writer)
}
```

### **4.2. 遙測: OpenTelemetry**

- **核心庫**: `opentelemetry`, `opentelemetry_sdk`, `tracing-opentelemetry`, `opentelemetry-otlp`, `opentelemetry-prometheus`
- **模塊**: `src/telemetry.rs`

#### 設計要點:

1.  **分佈式追踪 (Traces)**:
    - 使用 `opentelemetry-otlp` 將追踪數據導出到 OpenTelemetry Collector。
    - **跨邊界上下文傳遞**: 提供 `telemetry::context` 模塊，封裝 `inject_context` 和 `extract_context` 函數，用於在 HTTP Header 和消息隊列的消息中傳遞追踪上下文。模板中將提供一個在消息消費者中恢復追踪鏈路的範例。
2.  **指標 (Metrics)**:
    - 提供一個 `/metrics` 路由，通過 `opentelemetry-prometheus` 暴露給 Prometheus Server 抓取。
    - 提供一個全局的 `Meter` 實例，供業務代碼記錄自定義指標。

### **4.3. 錯誤處理**

- **核心模式**: 統一的 `AppError` 類型 + Axum 的 `IntoResponse` trait。
- **模塊**: `src/error.rs`

#### 設計要點:

1.  **統一錯誤類型**: 定義一個 `AppError` 枚舉，包含所有可能的業務和基礎設施錯誤。
2.  **無縫轉換**: 為 `AppError` 實現 `From<T>`，以便在代碼中通過 `?` 操作符將底層庫的錯誤（如 `sqlx::Error`）自動轉換。
3.  **標準化響應**: 為 `AppError` 實現 `IntoResponse`，將每個錯誤變體映射到一個具體的 HTTP 狀態碼和一個標準化的 JSON 響應體 (`{ "error_code": "...", "message": "..." }`)。

### **4.4. Panic 處理**

- **核心庫**: `std::panic`, `tokio`, `tracing`
- **模塊**: `src/telemetry.rs`

#### 設計要點:

1.  **全局 Panic Hook**: 服務啟動時，在 `main` 函數早期通過 `std::panic::set_hook` 設置了一個全局 panic hook。此 hook 作為最後一道防線，旨在捕獲所有未處理的 panic，確保服務的穩定性。
    - **日誌記錄**: 當 panic 發生時，hook 會使用 `tracing::error!` 記錄以下詳細信息。這些日誌遵循應用配置的結構化日誌格式 (默認為 JSON)，以便於後續分析和告警：
        - **Panic Payload**: panic 時傳遞的具體錯誤消息或數據。
        - **Panic Location**: 發生 panic 的源代碼位置 (包括文件名、行號和列號)，前提是 `PanicInfo` 結構能夠提供此類信息。
        - **Backtrace**: 詳細的函數調用堆棧追蹤。此信息在環境變量 `RUST_BACKTRACE=1` 被設置時由 `std::backtrace::Backtrace::capture()` 自動捕獲並記錄，極大地幫助了問題定位的深度。
    - **遙測數據刷寫 (Telemetry Data Flushing)**: (TODO) 理想情況下，panic hook 在記錄錯誤後，應嘗試優雅地關閉並刷寫所有緩存中的遙測數據（如追踪和指標），以確保在服務終止前最大限度地保留可觀測性數據。然而，由於在當前目標構建環境中遇到了 `opentelemetry::global::shutdown_tracer_provider()` 和 `shutdown_meter_provider()` 相關函數的編譯時解析錯誤 (E0425)，此遙測數據刷寫功能暫時被省略。待相關環境或依賴問題解決後，將重新審視並實現此功能。
    - 此 panic hook 的實現旨在滿足 T01.2.5 (Panic Handling and Reporting) 的核心要求，相關的配置和行為文檔（即本節更新）對應 T01.2.7 (Documentation of Panic Handling)。
2.  **安全的異步任務**:
    - **Tokio Task Panics**: 標準的 `std::panic::set_hook` 所設置的 hook 可能無法直接捕獲由 `tokio::spawn` 創建的異步任務內部發生的 panic，因為這些任務可能在不同的線程或上下文中運行。
    - **解決方案**: 提供一個 `telemetry::spawn_instrumented` 輔助函數。此函數在內部使用 `tracing::Instrument` 為每個異步任務附加一個 `Span`，並在任務完成時檢查其結果。如果任務 panic，則在此處捕獲並記錄錯誤。
3.  **文檔警告**: 模板的 `README.md` 中將明確警告開發者，必須使用 `spawn_instrumented` 來啟動所有背景任務，以確保 Panic 不會被“吞噬”。

### **4.5. 內置運維端點與特性**

- **模塊**: `src/handlers/`

1.  **/healthz**: 提供一個簡單的健康檢查端點，用於 K8s 的 liveness/readiness 探針。
2.  **/metrics**: Prometheus 指標端點。
3.  **/info**: 返回服務的構建信息，如 Git commit hash 和構建時間。使用 `vergen` crate 在編譯時自動注入這些信息。
4.  **限流 (Rate Limiting)**:
    - 通過 `tower_governor` 提供一個可選的請求限流中間件。
    - 可在配置文件中啟用並設置速率限制（如 `rate_limiter.requests_per_minute`）。

---

## **5. 模板結構與擴展性**

### **5.1. 目錄結構**

```text
service-template/
├── .github/workflows/ci.yml # 內置 CI 流程
├── scripts/                 # 輔助腳本
├── tests/                   # 整合測試
│   └── health_check.rs
├── Cargo.toml               # 使用 Feature Flags 管理依賴
├── Dockerfile.multistage    # 優化的多階段 Dockerfile
└── src/
    ├── main.rs
    ├── lib.rs
    ├── config.rs
    ├── error.rs
    ├── telemetry.rs           # 包含 context 和 panic 處理
    ├── middleware/            # 存放 Axum Layers, e.g., rate_limiter.rs
    └── handlers/              # API 處理器, e.g., healthz.rs, info.rs
```

### **5.2. Feature Gating**

為了保持模板的輕量和靈活性，將採用 Feature Flags 管理協議依賴：

```toml
# Cargo.toml
[features]
default = ["http"]
http    = ["axum", "tower-http", "tower_governor"]
grpc    = ["tonic", "prost"]
```

模板的 `main.rs` 和中間件將使用 `#[cfg(feature = "grpc")]` 等屬性來條件編譯不同協議的服務啟動代碼和追踪層。

---

## **6. CI/CD 與部署**

- **持續整合 (CI)**: 模板內置的 GitHub Actions workflow 將執行 `cargo check`, `fmt`, `clippy`, `test` 和 `cargo-deny` 安全審計。
- **容器化**: 提供一個優化的多階段 `Dockerfile`，使用 `musl` 進行靜態鏈接，最終鏡像基於 `scratch` 或 `distroless`，以最小化體積和攻擊面。
- **部署**: 推薦使用 Helm Chart 進行 Kubernetes 部署。模板文檔將提供一個標準 `values.yaml` 的建議結構，用於配置資源、副本數和遙測後端地址。

---
