# Contributing to Rust Service Scaffold

感謝您對本專案的貢獻興趣！本文檔提供了參與貢獻的詳細指南。

## 🚀 快速開始

### 前置要求

- [Rust toolchain](https://www.rust-lang.org/tools/install) (最新穩定版)
- [Docker & Docker Compose](https://docs.docker.com/get-docker/)
- Git

### 設置開發環境

1. **Fork 並克隆專案**
   ```bash
   git clone https://github.com/your-username/rust-service-scaffold.git
   cd rust-service-scaffold
   ```

2. **設置環境變數**
   ```bash
   cp .env.example .env
   ```

3. **啟動開發依賴**
   ```bash
   make docker-up
   ```

4. **運行開發伺服器**
   ```bash
   make dev
   ```

## 🏗️ 開發工作流程

### 分支策略

- `main`: 穩定的生產版本
- `develop`: 開發分支
- `feature/*`: 功能分支
- `fix/*`: 錯誤修復分支

### 提交流程

1. **創建功能分支**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **進行開發**
   - 遵循架構原則
   - 編寫測試
   - 更新文檔

3. **品質檢查**
   ```bash
   make quality-check
   ```

4. **提交變更**
   ```bash
   git add .
   git commit -m "feat(scope): description"
   ```

5. **推送並創建 PR**
   ```bash
   git push origin feature/your-feature-name
   ```

## 📋 貢獻檢查清單

### 程式碼品質
- [ ] 遵循六邊形架構原則
- [ ] 通過所有測試 (`make test`)
- [ ] 通過格式化檢查 (`cargo fmt --check`)
- [ ] 通過 Clippy 檢查 (`cargo clippy`)
- [ ] 通過安全審計 (`cargo audit`)

### 測試要求
- [ ] 新功能有對應的單元測試
- [ ] 整合測試覆蓋主要用例
- [ ] 測試覆蓋率符合要求
- [ ] 所有測試通過

### 文檔更新
- [ ] 更新 README.md (如需要)
- [ ] 更新 CHANGELOG.md
- [ ] 添加程式碼註釋和文檔
- [ ] 更新 API 文檔 (如需要)

## 🎯 架構指南

### 依賴規則
- Domain 層不能依賴任何外部 crate
- Application 層只能依賴 Domain 和 Contracts
- Infrastructure 層實現 Contracts 中的端口
- Presentation 層只能調用 Application 層

### 新增功能流程
1. 在 `contracts/src/ports.rs` 定義端口
2. 在 `domain/` 實現業務邏輯
3. 在 `application/src/use_cases/` 創建用例
4. 在 `infra_*/` 實現適配器
5. 在 `presentation/` 添加 API
6. 在 `bootstrap/src/factory.rs` 組裝依賴

## 🧪 測試策略

### 測試層級
- **單元測試**: 80% (每個 crate 內部)
- **整合測試**: 15% (bootstrap/tests/)
- **E2E 測試**: 5% (完整流程)

### 測試命令
```bash
# 運行所有測試
make test

# 運行特定層級測試
cargo test -p domain
cargo test -p application
cargo test --test integration_test
```

## 📝 提交訊息規範

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
type(scope): description

[optional body]

[optional footer]
```

### 類型
- `feat`: 新功能
- `fix`: 錯誤修復
- `docs`: 文檔更新
- `style`: 程式碼格式化
- `refactor`: 重構
- `test`: 測試相關
- `chore`: 建構工具或輔助工具變動

### 範例
```
feat(user): add user creation endpoint

- Implement CreateUserUseCase
- Add PostgreSQL user repository
- Add validation for user input
- Add integration tests

Closes #123
```

## 🔍 程式碼審查

### 審查重點
- 架構原則遵循
- 程式碼品質和可讀性
- 測試覆蓋和品質
- 性能影響
- 安全考量

### 審查流程
1. 自動化檢查通過
2. 至少一位維護者審查
3. 解決所有評論
4. 合併到目標分支

## 🐛 問題回報

### Bug 回報
使用 GitHub Issues，包含：
- 問題描述
- 重現步驟
- 預期行為
- 實際行為
- 環境資訊

### 功能請求
- 清楚描述需求
- 說明使用場景
- 提供可能的解決方案

## 📞 聯繫方式

- GitHub Issues: 技術問題和 Bug 回報
- GitHub Discussions: 一般討論和問題
- Pull Requests: 程式碼貢獻

感謝您的貢獻！🎉