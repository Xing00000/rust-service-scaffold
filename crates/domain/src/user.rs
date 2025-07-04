use uuid::Uuid;

//=== Domain Entity ===//
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let id = Uuid::now_v7();
        let name = "Test User".to_string();

        let user = User {
            id,
            name: name.clone(),
        };

        assert_eq!(user.id, id);
        assert_eq!(user.name, name);
    }

    #[test]
    fn test_user_clone() {
        let user = User {
            id: Uuid::now_v7(),
            name: "Original".to_string(),
        };

        let cloned = user.clone();
        assert_eq!(user.id, cloned.id);
        assert_eq!(user.name, cloned.name);
    }
}
