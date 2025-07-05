pub mod health;
pub mod main;
pub mod user;

// Re-export all handlers
pub use health::*;
pub use main::*;
pub use user::*;
