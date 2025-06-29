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

```
Presentation / Infrastructure --> Application --> Domain
```

- **Domain**: 核心業務邏輯和實體。最純淨的一層，不依賴任何外部框架。
- **Application**: 應用程式的用例 (Use Cases) 和端口 (Ports)。定義了應用「做什麼」，但不關心「如何做」。
- **Infrastructure**: 外部系統的具體實現 (Adapters)，如資料庫、快取、消息隊列。實現 Application 層定義的端口。
- **Presentation**: 對外暴露的介面 (Driving Adapters)，如 REST API、gRPC 或 CLI。

這種結構確保了核心業務邏輯的獨立性和可測試性，使得替換任何外部依賴（如 Web 框架或資料庫）都變得相對容易。

## 📁 目錄結構

```text
hexagonal_template/
├── Cargo.toml            # Workspace 根配置
├── Makefile              # 開發自動化命令
├── scripts/              # 常用腳本
│
├── crates/               # 核心 Library Crates
│   ├── domain/           # 核心業務邏輯、實體 (Entities)
│   ├── application/      # 用例 (Use Cases)、端口 (Ports)
│   ├── infra_db_postgres/# PostgreSQL 適配器
│   ├── infra_telemetry/  # 可觀測性 (日誌/指標/追踪) 適配器
│   └── ...               # 其他基礎設施適配器
│
├── presentation/         # 對外介面層
│   └── pres_web_axum/    # Axum Web API 的實現 (Handlers/Router/DTOs)
│
├── app/                  # Binary Crate (可執行檔)
│   └── src/main.rs       # 應用程式組裝點 (Composition Root)
│
├── config/               # 預設設定檔 (e.g., default.toml)
│
└── tests/                # 跨 Crate 的整合與端到端測試
```

## 🧪 測試策略

我們採用分層的測試策略，以確保程式碼品質和開發效率：

1.  **單元測試 (`#[cfg(test)]`)**:

    - **位置**: 在各個 crate 的 `src/` 目錄下。
    - **目標**: 測試單一模組或函數的邏輯，速度快，無 I/O。使用 mock 或 fake 物件來模擬依賴。

2.  **整合測試 (頂層 `tests/` 目錄)**:

    - **位置**: 專案根目錄下的 `tests/`。
    - **目標**: 測試 crate 之間的協作和公開 API 的行為。這些測試像外部用戶一樣調用 crate。

3.  **端到端測試 (E2E)**:
    - 整合測試的一種，會啟動完整的應用程式（或其輕量版本）和真實的外部依賴（如資料庫），來模擬真實的用戶場景。

## 🔧 配置 (Configuration)

應用程式的配置通過以下方式加載，優先級從低到高：

1.  **`config/default.toml`**: 存儲所有配置項的預設值。
2.  **環境變數**:
    - 可以通過 `.env` 文件設置。
    - 變數需以 `APP_` 為前綴，並使用 `__` 作為層級分隔符。例如，要覆蓋 `db.host`，需設置環境變數 `APP_DB__HOST=...`。

所有可配置的選項都在 `app/src/config.rs` 中定義。

## 📜 貢獻 (Contributing)

歡迎提交 Pull Requests！為了保持程式碼品質，請確保：

- 你的程式碼通過了 `cargo clippy --all-targets -- -D warnings` 的檢查。
- 你的程式碼通過了 `cargo fmt` 的格式化。
- 所有現有測試都能通過，並為新功能添加了適當的測試。

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
