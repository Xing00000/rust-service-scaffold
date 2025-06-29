// src/domain/mod.rs
// 包含核心業務邏輯、實體和領域特定的錯誤。
// 這一層不應該知道任何關於 Web 框架或數據庫的具體實現。

pub mod error;

// 將來可以在這裡定義業務實體, e.g.
// pub mod user;
