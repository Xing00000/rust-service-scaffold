use async_trait::async_trait;
use contracts::ports::{DomainError, ObservabilityPort, User, UserRepository};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use uuid::Uuid;

#[derive(Default)]
pub struct FakeUserRepository;

#[async_trait]
impl UserRepository for FakeUserRepository {
    async fn find(&self, id: &Uuid) -> Result<User, DomainError> {
        Ok(User {
            id: *id,
            name: "Test User".to_string(),
        })
    }

    async fn save(&self, _user: &User) -> Result<(), DomainError> {
        Ok(())
    }

    async fn shutdown(&self) {}
}

#[derive(Clone, Default)]
pub struct FakeObservability {
    request_start_calls: Arc<AtomicUsize>,
    request_end_calls: Arc<AtomicUsize>,
}

impl FakeObservability {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_request_start_calls(&self) -> usize {
        self.request_start_calls.load(Ordering::SeqCst)
    }

    pub fn get_request_end_calls(&self) -> usize {
        self.request_end_calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ObservabilityPort for FakeObservability {
    async fn on_request_start(&self, _method: &str, _path: &str) {
        self.request_start_calls.fetch_add(1, Ordering::SeqCst);
    }

    async fn on_request_end(&self, _method: &str, _path: &str, _status: u16, _latency: f64) {
        self.request_end_calls.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Clone, Default)]
pub struct TestWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl TestWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_logs(&self) -> String {
        String::from_utf8_lossy(&self.buf.lock().unwrap()).to_string()
    }
}

impl std::io::Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buf.lock().unwrap().flush()
    }
}
