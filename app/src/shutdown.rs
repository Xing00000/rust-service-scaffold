/// A trait for resources that need to be gracefully shut down.
#[async_trait::async_trait]
pub trait ShutdownHook: Send + Sync {
    /// Performs the shutdown logic for the resource.
    /// The `self` argument suggests that the hook might consume itself or operate on its state.
    /// If hooks are to be stored and called, they might need to be `Arc<dyn ShutdownHook>`
    /// or the method could take `&self` or `&mut self` if state is managed externally.
    /// For simplicity, let's start with `self` and adapt if needed.
    async fn shutdown(self);
}

// A version if we want to call shutdown on shared resources:
// #[async_trait::async_trait]
// pub trait SharedShutdownHook: Send + Sync {
//     async fn shutdown(&self);
// }

// For now, the `self` consuming version is fine as we can wrap components in Arc if needed
// and then implement the trait for the Arc-wrapped component if the inner component's
// shutdown method requires ownership or mutable access not suitable for &dyn.
// However, sqlx Pool::close is &self, and OTel global shutdown is a global call.

// Let's refine: The hook itself might be a wrapper or a manager.
// The state that needs to be shut down (like the Pool) is likely in AppState.
// So, the shutdown_signal function will likely take AppState.

// Alternative approach: The shutdown_signal calls specific shutdown functions.
// For instance, telemetry_shutdown(), db_pool_close(&app_state.user_repo.pool()).
// This might be simpler than a generic ShutdownHook trait if there are few items.

// The request was to "Create trait ShutdownHook ...將 telemetry::shutdown()、DB pool close 等放入統一關機流程。"
// This implies a collection of hooks.

// Let's consider that AppState itself could implement a method that calls all necessary shutdowns.
// Or, we collect Box<dyn ShutdownHook> during AppState creation.

// Let's try with a simple trait first, and implement it for wrappers around resources.
// The `shutdown_signal` can then iterate over a list of `Box<dyn ShutdownHook>`.
// This seems more aligned with the "trait" request.
// The items being shut down (Pool, OTel providers) might not be `self` consumed.
// So `async fn shutdown(&self)` or `async fn shutdown(&mut self)` might be more appropriate.
// Let's use `async fn shutdown(&self)` for now, as it's common for shutdown methods
// not to consume the resource entirely but to signal it to stop.

#[async_trait::async_trait]
pub trait ShutdownHookV2: Send + Sync {
    async fn shutdown(&self);
}

// This list will be populated during application setup.
pub type ShutdownHooks = Vec<Box<dyn ShutdownHookV2>>;

// Example: How OpenTelemetry global shutdown could be wrapped
pub struct OpenTelemetryShutdownHook;

#[async_trait::async_trait]
impl ShutdownHookV2 for OpenTelemetryShutdownHook {
    async fn shutdown(&self) {
        opentelemetry::global::shutdown_tracer_provider();
        // opentelemetry::global::shutdown_meter_provider(); // If available and needed
        tracing::info!("OpenTelemetry providers shut down.");
    }
}

use application::ports::UserRepository;
use std::sync::Arc;

// Wrapper for UserRepository shutdown
pub struct UserRepositoryShutdownHook {
    user_repo: Arc<dyn UserRepository>,
}

impl UserRepositoryShutdownHook {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }
}

#[async_trait::async_trait]
impl ShutdownHookV2 for UserRepositoryShutdownHook {
    async fn shutdown(&self) {
        self.user_repo.shutdown().await;
    }
}
