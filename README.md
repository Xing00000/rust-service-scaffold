# Rust Hexagonal Architecture Template

一個基於 Rust 和 Axum 的生產級後端服務樣板，嚴格遵循**六邊形架構 (Hexagonal Architecture / Ports and Adapters)** 和領域驅動設計 (DDD) 的思想。

這個樣板的目標是提供一個高內聚、低耦合、可測試、可演化的起點，幫助你快速構建健壯且可長期維護的後端應用。

[![CI](https://github.com/<YOUR_USERNAME>/<YOUR_REPO>/actions/workflows/ci.yml/badge.svg)](https://github.com/<YOUR_USERNAME>/<YOUR_REPO>/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ✨ 特性 (Features)

- **🧅 六邊形架構**: 清晰的 `domain`, `application`, `infrastructure`, `presentation` 分層。
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

- **Domain**: 核心業務邏輯和實體，完全獨立，無外部依賴
- **Contracts**: 統一的端口定義和共享類型，連接各層的抽象
- **Application**: 用例實現和依賴注入容器，協調業務流程
- **Infrastructure**: 外部系統適配器（資料庫、監控等）
- **Presentation**: 對外介面（REST API、CLI 等）
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

| 項目 | 重構前 | 重構後 |
|------|--------|--------|
| 端口定義 | 分散在各層 | 統一在 `contracts` |
| 依賴注入 | 手動組裝 | 工廠模式 + 容器 |
| 測試支援 | 不完整 | Mock + 單元測試 |
| 依賴方向 | 部分違反 | 嚴格遵循 |

### 🔗 新增 Contracts 層

統一管理所有抽象介面：

```rust
// contracts/src/ports.rs
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: &Uuid) -> Result<User, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
}

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
    let mut mock_repo = MockUserRepository::new();
    mock_repo.expect_save().returning(|_| Box::pin(async { Ok(()) }));
    
    let use_case = UserSvc::new(Arc::new(mock_repo));
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

## 🔧 開發指南

### 新增功能流程

1. **定義端口** - 在 `contracts/src/ports.rs` 添加抽象介面
2. **實現領域邏輯** - 在 `domain/` 添加業務實體和邏輯
3. **創建用例** - 在 `application/src/use_cases/` 實現業務流程
4. **實現適配器** - 在 `infra_*/` 實現具體技術細節
5. **添加 API** - 在 `presentation/` 暴露對外介面
6. **更新工廠** - 在 `bootstrap/src/factory.rs` 組裝依賴

### 程式碼品質檢查

```bash
# 格式化程式碼
cargo fmt

# 靜態分析
cargo clippy --all-targets -- -D warnings

# 執行測試
cargo test

# 檢查依賴
cargo check
```

### 架構原則

- ✅ **依賴方向**: 永遠從外向內
- ✅ **端口優先**: 先定義抽象，再實現具體
- ✅ **測試驅動**: 每個用例都有對應測試
- ✅ **單一職責**: 每層只關心自己的職責

## 📜 貢獻 (Contributing)

歡迎提交 Pull Requests！請確保：

- 遵循六邊形架構原則
- 通過所有品質檢查
- 添加適當的測試覆蓋
- 更新相關文件

## 📄 授權 (License)

本專案採用 [MIT License](LICENSE)。

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



