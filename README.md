# Rust Hexagonal Architecture Template

一個基於 Rust 和 Axum 的生產級後端服務樣板，嚴格遵循**六邊形架構 (Hexagonal Architecture / Ports and Adapters)** 和領域驅動設計 (DDD) 的思想。

這個樣板的目標是提供一個高內聚、低耦合、可測試、可演化的起點，幫助你快速構建健壯且可長期維護的後端應用。

[![CI](https://github.com/<YOUR_USERNAME>/<YOUR_REPO>/actions/workflows/ci.yml/badge.svg)](https://github.com/<YOUR_USERNAME>/<YOUR_REPO>/actions/workflows/ci.yml)

---

## ✨ 特性 (Features)

- **🧅 六邊形架構**: 清晰的 `domain`, `application`, `infrastructure`, `presentation` 分層，Domain 層完全零依賴。
- **📦 Cargo Workspace**: 強制模組邊界，加速編譯，提升專案組織性。
- **🚀 生產級 Web 服務**:
  - **Axum**: 高性能、符合人體工學的 Web 框架。
  - **優雅關閉 (Graceful Shutdown)**: 安全地處理 `Ctrl+C` 和 `SIGTERM` 信號。
  - **請求 ID**: 追蹤請求的完整生命週期。
  - **限流 (Rate Limiting)**: 使用 `tower-governor` 防止濫用。
- **🔭 全棧可觀測性 (Full-Stack Observability)**:
  - **結構化日誌 (Logging)**: 使用 `tracing` 進行 JSON 格式的結構化日誌記錄。
  - **指標 (Metrics)**: 使用 `prometheus` 導出關鍵服務指標。
  - **追踪 (Tracing)**: 集成 `opentelemetry` 實現分散式追踪。
  - **Panic Hook**: 捕獲未處理的 Panic 並以日誌形式記錄詳細信息。
- **🛡️ 健壯的錯誤處理**: 統一的錯誤類型，自動映射到結構化的 HTTP 響應。
- **⚙️ 靈活的配置管理**: 使用 `figment` 從文件和環境變數加載配置。
- **🧪 全面的測試策略**: 涵蓋單元測試、整合測試和端到端測試。
- **⚡ 開發者體驗優先**: 提供 `Makefile` 和腳本，簡化常見開發任務。

## 🚀 快速上手 (Quick Start)

### 前置要求

- [Rust toolchain](https://www.rust-lang.org/tools/install) (最新穩定版)
- Docker & Docker Compose (用於運行資料庫等依賴)

### 1. 克隆專案

```bash
git clone https://github.com/<YOUR_USERNAME>/<YOUR_REPO>.git
cd <YOUR_REPO>
```

### 2. 配置環境

複製預設的設定檔。你可以根據需要修改 `.env` 文件。

```bash
cp .env.example .env
```

### 3. 啟動依賴服務 (如 PostgreSQL)

```bash
docker-compose up -d
```

### 4. 運行開發伺服器

我們提供了方便的 Makefile 命令來啟動開發伺服器，它會監聽文件變更並自動重新加載。

```bash
make dev
# 或者直接運行腳本
# sh ./scripts/dev.sh
```

服務將在 `http://127.0.0.1:8080` (預設) 上啟動。

### 5. 運行測試

運行所有測試，包括單元測試和整合測試。

```bash
make test
# 或者直接運行腳本
# sh ./scripts/test.sh
```

## 🏗️ 架構概覽

本專案採用嚴格的六邊形架構，依賴關係永遠是**從外向內**。

```mermaid
graph TB
    subgraph "Presentation Layer"
        WEB["Web API (Axum)"]
        CLI["CLI Interface"]
    end

    subgraph "Infrastructure Layer"
        DB["PostgreSQL Adapter"]
        METRICS["Telemetry Adapter"]
        CACHE["Cache Adapter"]
    end

    subgraph "Application Layer"
        UC["Use Cases"]
        PORTS["Ports (Interfaces)"]
        CONTAINER["DI Container"]
    end

    subgraph "Contracts Layer"
        IFACES["Unified Interfaces"]
        TYPES["Shared Types"]
    end

    subgraph "Domain Layer"
        ENTITIES["Business Entities"]
        LOGIC["Business Logic"]
        ERRORS["Domain Errors"]
    end

    subgraph "Bootstrap Layer"
        FACTORY["Dependency Factory"]
        CONFIG["Configuration"]
        MAIN["Application Entry"]
    end

    %% Dependencies (outer to inner)
    WEB --> UC
    CLI --> UC
    DB --> PORTS
    METRICS --> PORTS
    CACHE --> PORTS

    UC --> ENTITIES
    PORTS --> IFACES
    CONTAINER --> UC

    IFACES --> ENTITIES
    TYPES --> ENTITIES

    FACTORY --> CONTAINER
    FACTORY --> DB
    FACTORY --> METRICS
    CONFIG --> FACTORY
    MAIN --> FACTORY

    classDef domain fill:#e1f5fe
    classDef application fill:#f3e5f5
    classDef infrastructure fill:#fff3e0
    classDef presentation fill:#e8f5e8
    classDef contracts fill:#fce4ec
    classDef bootstrap fill:#f1f8e9

    class ENTITIES,LOGIC,ERRORS domain
    class UC,PORTS,CONTAINER application
    class DB,METRICS,CACHE infrastructure
    class WEB,CLI presentation
    class IFACES,TYPES contracts
    class FACTORY,CONFIG,MAIN bootstrap
```

### 🎯 架構層級說明

- **Domain**: 核心業務邏輯、實體和 **Port 定義**，完全零外部依賴
- **Contracts**: 重用 Domain 的 Port 定義，提供跨層的統一抽象
- **Application**: 用例實現、ID 轉換和依賴注入容器
- **Infrastructure**: 實現 Domain Port，適配外部系統
- **Presentation**: 對外介面，負責 HTTP 狀態碼映射
- **Bootstrap**: 應用程式組裝和啟動邏輯

### 🔄 依賴注入流程

```mermaid
sequenceDiagram
    participant Main as Bootstrap::Main
    participant Factory as DependencyFactory
    participant Container as Application::Container
    participant UseCase as Use Cases
    participant Adapter as Infrastructure

    Main->>Factory: create_container(config)
    Factory->>Adapter: create adapters
    Factory->>Container: new(adapters)
    Container->>UseCase: inject dependencies
    Main->>+Container: start application
```

## 📁 目錄結構

```text
rust-service-scaffold/
├── Cargo.toml                    # Workspace 根配置
├── Makefile                      # 開發自動化命令
├── docker-compose.yml            # 開發環境依賴
│
├── crates/                       # 核心 Library Crates
│   ├── contracts/                # 🔗 統一端口定義層
│   │   ├── src/ports.rs         # 所有抽象介面
│   │   └── src/error.rs         # 共享錯誤類型
│   ├── domain/                   # 🏛️ 領域層 (最內層)
│   │   ├── src/user.rs          # 業務實體
│   │   └── src/error.rs         # 領域錯誤
│   ├── application/              # 🎯 應用層
│   │   ├── src/use_cases/       # 用例實現
│   │   ├── src/container.rs     # 依賴注入容器
│   │   └── src/error.rs         # 應用錯誤
│   ├── infra_db_postgres/        # 🗄️ 資料庫適配器
│   └── infra_telemetry/          # 📊 監控適配器
│
├── presentation/                 # 🌐 表現層
│   └── pres_web_axum/           # Axum Web API
│       ├── src/handlers.rs      # HTTP 處理器
│       ├── src/dtos.rs          # 資料傳輸物件
│       └── src/middleware/      # 中介軟體
│
├── bootstrap/                    # 🚀 啟動層
│   ├── src/main.rs              # 應用程式入口
│   ├── src/factory.rs           # 依賴工廠
│   ├── src/config.rs            # 配置管理
│   └── tests/                   # 整合測試
│
└── config/                       # ⚙️ 配置檔案
    └── default.toml             # 預設配置
```

## 🧪 測試策略

採用分層測試策略，確保各層獨立可測：

```mermaid
pyramid
    title Test Pyramid

    "E2E Tests" : 5
    "Integration Tests" : 15
    "Unit Tests" : 80
```

### 測試層級

1. **單元測試** (`#[cfg(test)]`)

   - **Domain**: 純業務邏輯測試
   - **Application**: 使用 Mock 測試用例
   - **Infrastructure**: 適配器邏輯測試

2. **整合測試** (`bootstrap/tests/`)

   - HTTP API 端到端測試
   - 錯誤處理測試
   - 中介軟體測試

3. **契約測試**
   - Mock 自動生成 (`mockall`)
   - 端口介面驗證

### 執行測試

```bash
# 執行所有測試
cargo test

# 執行特定層級測試
cargo test -p application  # 應用層測試
cargo test -p domain      # 領域層測試

# 執行整合測試
cargo test --test integration_test
```

## 🔄 重構成果

本專案已經過完整的六邊形架構重構，主要改進包括：

### 🎯 架構改進

| 項目     | 重構前     | 重構後             |
| -------- | ---------- | ------------------ |
| Domain 依賴 | 依賴外部 crate | **完全零依賴** |
| Port 定義 | 分散在各層 | **Domain 單一定義** |
| ID 類型 | 裸 UUID | **封裝的 UserId** |
| 錯誤處理 | thiserror | **純 Rust 標準庫** |
| HTTP 映射 | contracts 層 | **presentation 層** |

### 🔗 Contracts 層重新定位

重用 Domain 層的 Port 定義，提供統一抽象：

```rust
// domain/src/ports.rs - 唯一的 Port 定義
pub trait UserRepository: Send + Sync {
    fn find(&self, id: &UserId) -> Pin<Box<dyn Future<Output = Result<User, DomainError>> + Send + '_>>;
    fn save(&self, user: &User) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>>;
}

// contracts/src/ports.rs - 重用 Domain 定義
pub use domain::UserRepository;

// 應用層特有的 Port
pub trait ObservabilityPort: Send + Sync {
    async fn on_request_start(&self, method: &str, path: &str);
    async fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64);
}
```

### 🏠 依賴注入容器

統一管理所有依賴：

```rust
// application/src/container.rs
pub struct Container {
    pub user_repo: DynUserRepo,
    pub observability: DynObservability,
    pub create_user_uc: Arc<dyn CreateUserUseCase>,
}
```

### 🏭 依賴工廠

負責組裝所有依賴：

```rust
// bootstrap/src/factory.rs
impl DependencyFactory {
    pub async fn create_container(config: &Config) -> Result<Container, Error> {
        let user_repo = Self::create_user_repository(config).await?;
        let observability = Self::create_observability();
        Ok(Container::new(user_repo, observability))
    }
}
```

### 🧪 測試改進

完整的 Mock 測試支援：

```rust
#[tokio::test]
async fn test_create_user_success() {
    struct MockUserRepository;
    
    impl UserRepository for MockUserRepository {
        fn save(&self, _user: &User) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
    }

    let use_case = UserSvc::new(Arc::new(MockUserRepository));
    let result = use_case.exec(cmd).await;
    assert!(result.is_ok());
}
```

## 🔧 配置 (Configuration)

應用程式的配置通過以下方式加載，優先級從低到高：

1.  **`config/default.toml`**: 存儲所有配置項的預設值。
2.  **環境變數**:
    - 可以通過 `.env` 文件設置。
    - 變數需以 `APP_` 為前綴，並使用 `__` 作為層級分隔符。例如，要覆蓋 `db.host`，需設置環境變數 `APP_DB__HOST=...`。

所有可配置的選項都在 `app/src/config.rs` 中定義。

## 📋 開發規範 (Development Guidelines)

### 🏗️ 架構原則 (Architecture Principles)

#### 核心設計原則

- **依賴倒置**: 所有依賴關係永遠從外向內 (Domain ← Application ← Infrastructure/Presentation)
- **端口優先**: 先定義抽象介面 (trait)，再實現具體適配器
- **單一職責**: 每層只關心自己的職責，避免跨層邏輯
- **測試驅動**: 每個用例都必須有對應的單元測試和整合測試
- **契約設計**: 使用 `contracts` 層統一管理所有抽象介面

#### 依賴規則檢查清單

- ✅ Domain 層**完全零外部依賴**，定義所有業務 Port
- ✅ Application 層只能依賴 Domain，負責 ID 轉換和用例編排
- ✅ Infrastructure 層**直接實現 Domain Port**，不重複定義
- ✅ Presentation 層負責 HTTP 狀態碼映射，不洩漏到其他層
- ✅ Contracts 層**只重用 Domain 定義**，不創建新概念

### 🔄 開發工作流程 (Development Workflow)

#### 新增功能的標準流程

1. **📋 需求分析**

   ```bash
   # 創建功能分支
   git checkout -b feature/user-management
   ```

2. **🎯 定義端口** (Contracts Layer)

   ```rust
   // contracts/src/ports.rs
   #[async_trait]
   pub trait NewFeatureRepository: Send + Sync {
       async fn operation(&self, param: &Type) -> Result<Output, DomainError>;
   }
   ```

3. **🏛️ 實現領域邏輯** (Domain Layer)

   ```rust
   // domain/src/new_entity.rs
   pub struct NewEntity {
       // 純業務邏輯，無外部依賴
   }
   ```

4. **🎯 創建用例** (Application Layer)

   ```rust
   // application/src/use_cases/new_feature.rs
   pub struct NewFeatureUseCase {
       repo: DynNewFeatureRepo,
   }
   ```

5. **🔌 實現適配器** (Infrastructure Layer)

   ```rust
   // infra_*/src/new_adapter.rs
   impl NewFeatureRepository for ConcreteAdapter {
       // 具體技術實現
   }
   ```

6. **🌐 添加 API** (Presentation Layer)

   ```rust
   // presentation/*/src/handlers.rs
   pub async fn new_feature_handler() {
       // HTTP/gRPC 處理器
   }
   ```

7. **🏭 更新工廠** (Bootstrap Layer)

   ```rust
   // bootstrap/src/factory.rs
   impl DependencyFactory {
       fn create_new_feature_adapter() -> DynNewFeatureRepo {
           // 依賴組裝
       }
   }
   ```

8. **🧪 編寫測試**

   ```bash
   # 單元測試
   cargo test -p domain
   cargo test -p application

   # 整合測試
   cargo test --test integration_test
   ```

### 🧪 測試規範 (Testing Standards)

#### 測試分層策略

```mermaid
pyramid
    title Test Pyramid
    "E2E Tests (5%)" : 5
    "Integration Tests (15%)" : 15
    "Unit Tests (80%)" : 80
```

#### 測試類型與要求

1. **單元測試** (每個 crate 內部)

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use mockall::predicate::*;

       #[tokio::test]
       async fn test_use_case_success() {
           // Arrange
           let mut mock_repo = MockRepository::new();
           mock_repo.expect_save()
               .with(eq(expected_input))
               .returning(|_| Box::pin(async { Ok(()) }));

           // Act
           let result = use_case.execute(input).await;

           // Assert
           assert!(result.is_ok());
       }
   }
   ```

2. **整合測試** (bootstrap/tests/)

   ```rust
   #[tokio::test]
   async fn test_api_endpoint() {
       let app = create_test_app().await;
       let response = app.oneshot(request).await.unwrap();
       assert_eq!(response.status(), StatusCode::OK);
   }
   ```

3. **契約測試** (自動生成 Mock)
   ```rust
   #[cfg_attr(test, mockall::automock)]
   pub trait Repository: Send + Sync {
       async fn find(&self, id: &Uuid) -> Result<Entity, Error>;
   }
   ```

#### 測試覆蓋率要求

- **Domain Layer**: 100% 覆蓋率 (純業務邏輯)
- **Application Layer**: 95% 覆蓋率 (用例邏輯)
- **Infrastructure Layer**: 80% 覆蓋率 (適配器邏輯)
- **Presentation Layer**: 85% 覆蓋率 (API 處理器)

### 🔧 程式碼品質標準 (Code Quality Standards)

#### 自動化檢查流程

```bash
# 完整品質檢查流程
make quality-check

# 或分步執行
cargo fmt --all --check          # 格式化檢查
cargo clippy --all-targets -- -D warnings  # 靜態分析
cargo test --workspace           # 執行所有測試
cargo audit                      # 安全漏洞檢查
```

#### Clippy 規則配置

```toml
# Cargo.toml - workspace.lints
[workspace.lints.rust]
unused = "deny"
rust_2018_idioms = "deny"
unused_imports = "deny"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

#### 程式碼風格要求

1. **命名規範**

   ```rust
   // ✅ 正確
   pub struct UserRepository;           // PascalCase for types
   pub fn create_user() -> Result<>;    // snake_case for functions
   const MAX_CONNECTIONS: u32 = 100;    // SCREAMING_SNAKE_CASE for constants

   // ❌ 錯誤
   pub struct userRepository;           // 應使用 PascalCase
   pub fn CreateUser() -> Result<>;     // 應使用 snake_case
   ```

2. **錯誤處理**

   ```rust
   // ✅ 使用純 Rust 標準庫定義結構化錯誤
   #[derive(Debug, Clone, PartialEq)]
   pub enum DomainError {
       NotFound { message: String },
       ValidationError { message: String },
       BusinessRule { message: String },
       InvalidOperation { message: String },
   }
   
   impl std::fmt::Display for DomainError {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           match self {
               DomainError::NotFound { message } => write!(f, "Entity not found: {}", message),
               DomainError::ValidationError { message } => write!(f, "Validation error: {}", message),
               // ...
           }
       }
   }

   // ✅ 使用 Result 類型和純 Future
   pub trait UserRepository: Send + Sync {
       fn find(&self, id: &UserId) -> Pin<Box<dyn Future<Output = Result<User, DomainError>> + Send + '_>>;
   }
   ```

3. **文檔要求**
   ````rust
   /// 用戶儲存庫端口定義
   ///
   /// 提供用戶實體的持久化操作抽象介面。
   /// 所有實現都必須保證操作的原子性和一致性。
   ///
   /// # Examples
   ///
   /// ```rust
   /// let user = repo.find(&user_id).await?;
   /// ```
   #[async_trait]
   pub trait UserRepository: Send + Sync {
       /// 根據 ID 查找用戶
       ///
       /// # Arguments
       ///
       /// * `id` - 用戶唯一標識符
       ///
       /// # Returns
       ///
       /// 成功時返回用戶實體，失敗時返回領域錯誤
       async fn find(&self, id: &Uuid) -> Result<User, DomainError>;
   }
   ````

### 🚀 性能優化指南 (Performance Guidelines)

#### 編譯優化

```toml
# Cargo.toml - 生產環境配置
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

#### 異步最佳實踐

```rust
// ✅ 使用 Arc 共享狀態
type DynRepository = Arc<dyn Repository>;

// ✅ 避免不必要的 clone
pub async fn process_batch(items: &[Item]) -> Result<Vec<Output>, Error> {
    let futures = items.iter().map(|item| process_item(item));
    try_join_all(futures).await
}

// ✅ 使用 tokio::spawn 處理 CPU 密集任務
tokio::task::spawn_blocking(move || {
    // CPU 密集計算
}).await?
```

### 🔒 安全規範 (Security Guidelines)

#### 輸入驗證

```rust
#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(range(min = 18, max = 120))]
    pub age: u8,
}
```

#### 敏感資料處理

```rust
// ✅ 使用 secrecy crate 處理敏感資料
use secrecy::{Secret, ExposeSecret};

pub struct DatabaseConfig {
    pub host: String,
    pub password: Secret<String>,
}

// ❌ 避免在日誌中洩露敏感資料
tracing::info!("Database config: {:?}", config); // 可能洩露密碼
```

### 📊 監控與可觀測性 (Observability)

#### 結構化日誌

```rust
// ✅ 使用結構化日誌
tracing::info!(
    user_id = %user.id,
    action = "user_created",
    "User successfully created"
);

// ✅ 使用 span 追蹤請求
#[tracing::instrument(skip(repo))]
pub async fn create_user(
    repo: &dyn UserRepository,
    request: CreateUserRequest,
) -> Result<User, Error> {
    // 實現邏輯
}
```

#### 指標收集

```rust
// 定義業務指標
static USER_OPERATIONS: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        Opts::new("user_operations_total", "Total user operations"),
        &["operation", "status"]
    ).unwrap()
});

// 記錄指標
USER_OPERATIONS.with_label_values(&["create", "success"]).inc();
```

### 🔄 CI/CD 整合 (CI/CD Integration)

#### GitHub Actions 工作流程

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --workspace
      - run: cargo audit
```

#### 發布檢查清單

- [ ] 所有測試通過
- [ ] 程式碼格式化檢查通過
- [ ] Clippy 靜態分析無警告
- [ ] 安全漏洞掃描通過
- [ ] 文檔更新完成
- [ ] 版本號更新
- [ ] CHANGELOG 更新

## 🔍 專案分析報告 (Project Analysis)

### 📊 程式碼統計

```
總計 Crates: 7 個
├── 核心層 (Core): 3 個
│   ├── domain/          - 領域層 (純業務邏輯)
│   ├── application/     - 應用層 (用例編排)
│   └── contracts/       - 契約層 (端口定義)
├── 基礎設施層: 2 個
│   ├── infra_db_postgres/    - PostgreSQL 適配器
│   └── infra_telemetry/      - 監控適配器
├── 表現層: 1 個
│   └── pres_web_axum/        - Axum Web API
└── 啟動層: 1 個
    └── bootstrap/            - 應用程式入口
```

### 🎯 架構亮點

1. **完全純淨的 Domain 層**

   - Domain 層實現真正的零外部依賴
   - 使用純 Rust 標準庫實現所有功能
   - Domain 層是所有 Port 定義的唯一來源

2. **完善的依賴注入**

   ```rust
   // 工廠模式組裝依賴
   pub struct DependencyFactory;

   impl DependencyFactory {
       pub async fn create_container(config: &Config) -> Result<Container, Error> {
           let user_repo = Self::create_user_repository(config).await?;
           let observability = Self::create_observability(config); // 配置化服務名
           Ok(Container::new(user_repo, observability))
       }
   }
   ```

3. **生產級可觀測性**

   - 結構化日誌 (JSON 格式)
   - Prometheus 指標導出
   - OpenTelemetry 分散式追踪
   - 自定義 Panic Hook

4. **全面的測試策略**
   - 自動生成 Mock (mockall)
   - 分層測試 (單元 + 整合 + E2E)
   - 測試覆蓋率要求明確

### 🔧 技術棧分析

#### 核心依賴

```toml
# Web 框架
axum = "0.8.4"              # 高性能 Web 框架
tower = "0.4.13"            # 中介軟體生態
tower-http = "0.5.2"        # HTTP 中介軟體

# 異步運行時
tokio = "1"                 # 異步運行時

# 資料庫
sqlx = "0.8.6"              # 異步 SQL 工具包

# 可觀測性
tracing = "0.1.40"          # 結構化日誌
prometheus = "0.13.3"       # 指標收集
opentelemetry = "0.28.0"    # 分散式追踪

# 配置管理
figment = "0.10"            # 靈活配置載入

# 錯誤處理
thiserror = "1.0.58"        # 結構化錯誤定義
```

#### 開發工具

```toml
# 測試工具
mockall = "0.12"            # Mock 生成

# 程式碼品質
clippy                      # 靜態分析
rustfmt                     # 程式碼格式化
cargo-audit                 # 安全漏洞檢查
```

### 📈 性能特性

1. **編譯時優化**

   - LTO (Link-Time Optimization) 啟用
   - 單一編譯單元 (codegen-units = 1)
   - 符號剝離 (strip = true)

2. **運行時性能**

   - 零成本抽象 (Rust 特性)
   - 異步 I/O (Tokio)
   - 連接池管理 (SQLx)
   - 限流保護 (tower-governor)

3. **記憶體安全**
   - 編譯時借用檢查
   - 無垃圾回收開銷
   - 自動記憶體管理

### 🛡️ 安全特性

1. **輸入驗證**

   ```rust
   #[derive(Deserialize, Validate)]
   pub struct CreateUserRequest {
       #[validate(length(min = 1, max = 100))]
       pub name: String,
       #[validate(email)]
       pub email: String,
   }
   ```

2. **錯誤處理**

   - 結構化錯誤類型
   - 不洩露內部實現細節
   - 統一錯誤響應格式

3. **限流保護**
   - 基於 IP 的請求限制
   - 可配置的限流參數
   - 優雅的限流響應

### 🚀 部署就緒特性

1. **容器化支援**

   - Docker Compose 開發環境
   - 多階段構建優化
   - 健康檢查端點

2. **配置管理**

   - 環境變數覆蓋
   - 配置驗證
   - 敏感資料保護

3. **優雅關閉**
   - SIGTERM 信號處理
   - 連接池清理
   - 請求完成等待

## 📜 貢獻 (Contributing)

### 🤝 貢獻流程

1. **Fork 專案**

   ```bash
   git clone https://github.com/your-username/rust-service-scaffold.git
   cd rust-service-scaffold
   ```

2. **創建功能分支**

   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **遵循開發規範**

   - 嚴格遵循六邊形架構原則
   - 添加適當的測試覆蓋 (最低 80%)
   - 更新相關文件
   - 通過所有品質檢查

4. **提交 Pull Request**
   - 提供清晰的變更說明
   - 包含測試結果截圖
   - 確保 CI 檢查通過

### ✅ 貢獻檢查清單

- [ ] 遵循架構分層原則
- [ ] 添加單元測試和整合測試
- [ ] 通過 `cargo fmt --check`
- [ ] 通過 `cargo clippy -- -D warnings`
- [ ] 通過 `cargo test --workspace`
- [ ] 通過 `cargo audit`
- [ ] 更新 CHANGELOG.md
- [ ] 更新相關文件

### 🏷️ 提交訊息規範

```
type(scope): description

[optional body]

[optional footer]
```

**類型 (type):**

- `feat`: 新功能
- `fix`: 錯誤修復
- `docs`: 文件更新
- `style`: 程式碼格式化
- `refactor`: 重構
- `test`: 測試相關
- `chore`: 建構工具或輔助工具的變動

**範例:**

```
feat(user): add user creation endpoint

- Implement CreateUserUseCase
- Add PostgreSQL user repository
- Add validation for user input
- Add integration tests

Closes #123
```

### Q: 如何添加新的資料庫支援？

**A:** 按照以下步驟：

1. 在 `contracts/src/ports.rs` 中定義儲存庫介面
2. 創建新的 `infra_db_*` crate
3. 實現儲存庫介面
4. 在 `bootstrap/src/factory.rs` 中添加工廠方法
5. 更新配置和測試

### Q: 如何進行性能調優？

**A:** 建議的調優步驟：

1. 使用 `cargo flamegraph` 進行性能分析
2. 檢查資料庫查詢效率
3. 優化異步任務調度
4. 調整連接池大小
5. 啟用編譯時優化選項

### Q: 如何處理跨服務事務？

**A:** 推薦使用以下模式：

- **Saga 模式**: 將長事務分解為多個步驟
- **事件溯源**: 記錄所有狀態變更事件
- **最終一致性**: 接受短暫的不一致狀態
- **補償操作**: 為每個操作定義回滾邏輯

```
find . -path ./target -prune -o -type f \( -name "*.rs" -o -name "*.toml" \) -print | while read file; do
  echo "=== $file ===" >> all_code.txt
  cat "$file" >> all_code.txt
  echo -e "\n" >> all_code.txt
done

```

```
docker run --name my-postgres \
  -e POSTGRES_USER=myuser \
  -e POSTGRES_PASSWORD=mypassword \
  -e POSTGRES_DB=mydb \
  -p 5432:5432 \
  -d postgres:16.9-bullseye

```

export DATABASE_URL="postgres://myuser:mypassword@localhost:5432/mydb"

psql postgres://myuser:mypassword@localhost:5432/mydb -c '\dt'
