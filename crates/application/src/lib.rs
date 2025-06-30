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
// src/application/mod.rs
// 包含應用服務（用例）和端口（接口定義）。
// 這一層協調 Domain 層的對象來執行業務操作。

pub mod error;
pub mod ports;
// pub mod use_cases; // 範例：將來可以有具體的用例模塊
