# 開發規範 (Development Guidelines)

## 🏗️ 架構原則 (Architecture Principles)

### 核心設計原則

- **依賴倒置**: 所有依賴關係永遠從外向內 (Domain ← Application ← Infrastructure/Presentation)
- **端口優先**: 先定義抽象介面 (trait)，再實現具體適配器
- **單一職責**: 每層只關心自己的職責，避免跨層邏輯
- **測試驅動**: 每個用例都必須有對應的單元測試和整合測試
- **契約設計**: 使用 `contracts` 層統一管理所有抽象介面

### 依賴規則檢查清單

- ✅ Domain 層**完全零外部依賴**，定義所有業務 Port
- ✅ Application 層只能依賴 Domain，負責 ID 轉換和用例編排
- ✅ Infrastructure 層**直接實現 Domain Port**，不重複定義
- ✅ Presentation 層負責 HTTP 狀態碼映射，不洩漏到其他層
- ✅ Contracts 層**只重用 Domain 定義**，不創建新概念

## 🔄 開發工作流程 (Development Workflow)

### 新增功能的標準流程

1. **📋 需求分析**
   ```bash
   git checkout -b feature/user-management
   ```

2. **🎯 定義端口** (Domain Layer)
   ```rust
   // domain/src/ports.rs
   pub trait NewFeatureRepository: Send + Sync {
       fn operation(&self, param: &Type) -> Pin<Box<dyn Future<Output = Result<Output, DomainError>> + Send + '_>>;
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
       repo: Arc<dyn NewFeatureRepository>,
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
       fn create_new_feature_adapter() -> Arc<dyn NewFeatureRepository> {
           // 依賴組裝
       }
   }
   ```

8. **🧪 編寫測試**
   ```bash
   cargo test -p domain
   cargo test -p application
   cargo test --test integration_test
   ```

## 🧪 測試規範 (Testing Standards)

### 測試分層策略

- **單元測試 (80%)**: 每個 crate 內部測試
- **整合測試 (15%)**: bootstrap/tests/ 中的 API 測試
- **E2E 測試 (5%)**: 完整流程測試

### 測試類型與要求

#### 1. 單元測試

每個功能必須包含三種測試場景：**正常場景**、**異常場景**、**邊界場景**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 正常場景測試
    #[tokio::test]
    async fn test_create_user_success() {
        // Arrange
        struct MockRepository;
        impl UserRepository for MockRepository {
            fn save(&self, _: &User) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>> {
                Box::pin(async { Ok(()) })
            }
        }

        let use_case = CreateUserUseCase::new(Arc::new(MockRepository));
        let cmd = CreateUserCommand {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };

        // Act
        let result = use_case.execute(cmd).await;

        // Assert
        assert!(result.is_ok());
    }

    // 異常場景測試
    #[tokio::test]
    async fn test_create_user_repository_error() {
        // Arrange
        struct FailingRepository;
        impl UserRepository for FailingRepository {
            fn save(&self, _: &User) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>> {
                Box::pin(async { Err(DomainError::InvalidOperation { message: "Database error".to_string() }) })
            }
        }

        let use_case = CreateUserUseCase::new(Arc::new(FailingRepository));
        let cmd = CreateUserCommand {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };

        // Act
        let result = use_case.execute(cmd).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::InvalidOperation { .. }));
    }

    // 邊界場景測試
    #[tokio::test]
    async fn test_create_user_boundary_conditions() {
        struct MockRepository;
        impl UserRepository for MockRepository {
            fn save(&self, _: &User) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>> {
                Box::pin(async { Ok(()) })
            }
        }

        let use_case = CreateUserUseCase::new(Arc::new(MockRepository));

        // 測試最小長度名稱
        let cmd_min = CreateUserCommand {
            name: "A".to_string(),
            email: "a@b.co".to_string(),
        };
        assert!(use_case.execute(cmd_min).await.is_ok());

        // 測試最大長度名稱
        let cmd_max = CreateUserCommand {
            name: "A".repeat(100),
            email: "test@example.com".to_string(),
        };
        assert!(use_case.execute(cmd_max).await.is_ok());

        // 測試空名稱（應該失敗）
        let cmd_empty = CreateUserCommand {
            name: "".to_string(),
            email: "test@example.com".to_string(),
        };
        assert!(use_case.execute(cmd_empty).await.is_err());
    }
}
```

#### 2. 整合測試
```rust
#[tokio::test]
async fn test_api_endpoint() {
    let app = create_test_app().await;
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

### 測試覆蓋率要求

- **Domain Layer**: 100% 覆蓋率 (純業務邏輯)
- **Application Layer**: 95% 覆蓋率 (用例邏輯)
- **Infrastructure Layer**: 80% 覆蓋率 (適配器邏輯)
- **Presentation Layer**: 85% 覆蓋率 (API 處理器)

### 測試場景要求

每個功能必須包含以下三種測試場景：

- **正常場景 (Happy Path)**: 測試正常輸入和預期行為
- **異常場景 (Error Cases)**: 測試錯誤處理和失敗情況
- **邊界場景 (Edge Cases)**: 測試極限值、空值、異常輸入

## 🔧 程式碼品質標準 (Code Quality Standards)

### 自動化檢查流程

```bash
# 完整品質檢查
make quality-check

# 分步執行
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo audit
```

### 程式碼風格要求

#### 1. 命名規範
```rust
// ✅ 正確
pub struct UserRepository;           // PascalCase for types
pub fn create_user() -> Result<>;    // snake_case for functions
const MAX_CONNECTIONS: u32 = 100;    // SCREAMING_SNAKE_CASE for constants

// ❌ 錯誤
pub struct userRepository;           // 應使用 PascalCase
pub fn CreateUser() -> Result<>;     // 應使用 snake_case
```

#### 2. 錯誤處理
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

#### 3. 文檔要求
```rust
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
    fn find(&self, id: &UserId) -> Pin<Box<dyn Future<Output = Result<User, DomainError>> + Send + '_>>;
}
```

## 🚀 性能優化指南 (Performance Guidelines)

### 編譯優化
```toml
# Cargo.toml - 生產環境配置
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

### 異步最佳實踐
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

## 🔒 安全規範 (Security Guidelines)

### 輸入驗證
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

### 敏感資料處理
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

## 📊 監控與可觀測性 (Observability)

### 結構化日誌
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

### 指標收集
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

## 🔄 CI/CD 整合 (CI/CD Integration)

### GitHub Actions 工作流程
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

### 發布檢查清單

- [ ] 所有測試通過
- [ ] 程式碼格式化檢查通過
- [ ] Clippy 靜態分析無警告
- [ ] 安全漏洞掃描通過
- [ ] 文檔更新完成
- [ ] 版本號更新
- [ ] CHANGELOG 更新

## 🤝 貢獻流程 (Contributing)

### 貢獻檢查清單

- [ ] 遵循架構分層原則
- [ ] 添加單元測試和整合測試
- [ ] 通過 `cargo fmt --check`
- [ ] 通過 `cargo clippy -- -D warnings`
- [ ] 通過 `cargo test --workspace`
- [ ] 通過 `cargo audit`
- [ ] 更新 CHANGELOG.md
- [ ] 更新相關文件

### 提交訊息規範

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

## 📚 常見問題 (FAQ)

### Q: 如何添加新的資料庫支援？

**A:** 按照以下步驟：
1. 在 `domain/src/ports.rs` 中定義儲存庫介面
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