#![deny(
    bad_style,
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
// src/domain/mod.rs
// 包含核心業務邏輯、實體和領域特定的錯誤。
// 這一層不應該知道任何關於 Web 框架或數據庫的具體實現。

pub mod error;
pub mod user;
