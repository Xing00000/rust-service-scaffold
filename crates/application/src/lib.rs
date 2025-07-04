#![deny(
    bad_style,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]

pub mod container;
pub mod error;
pub mod use_cases;

// Re-export contracts for convenience
pub use container::*;
pub use contracts::ports::*;
