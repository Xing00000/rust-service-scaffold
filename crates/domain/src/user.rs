use uuid::Uuid;

//=== Domain Entity ===//
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}
