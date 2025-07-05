//=== Pure Domain ID Types ===//

/// 用戶唯一標識符 - 純領域類型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    /// 從字符串創建 UserId（由 Application 層調用）
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// 獲取內部字符串值
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 轉換為字符串
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_creation() {
        let id_str = "test-id-123".to_string();
        let user_id = UserId::from_string(id_str.clone());

        assert_eq!(user_id.as_str(), "test-id-123");
        assert_eq!(user_id.into_string(), id_str);
    }

    #[test]
    fn test_user_id_display() {
        let user_id = UserId::from_string("display-test".to_string());
        assert_eq!(format!("{}", user_id), "display-test");
    }

    #[test]
    fn test_user_id_equality() {
        let id1 = UserId::from_string("same-id".to_string());
        let id2 = UserId::from_string("same-id".to_string());
        let id3 = UserId::from_string("different-id".to_string());

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
