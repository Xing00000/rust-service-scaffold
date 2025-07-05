//=== ID Generation Service ===//

use domain::UserId;
use uuid::Uuid;

/// ID 生成服務 - 負責將外部 UUID 轉換為領域 ID
pub struct IdService;

impl IdService {
    /// 生成新的用戶 ID
    pub fn generate_user_id() -> UserId {
        let uuid = Uuid::now_v7();
        UserId::from_string(uuid.to_string())
    }

    /// 從 UUID 字符串創建用戶 ID
    pub fn user_id_from_uuid_string(uuid_str: &str) -> Result<UserId, uuid::Error> {
        let _uuid = Uuid::parse_str(uuid_str)?; // 驗證格式
        Ok(UserId::from_string(uuid_str.to_string()))
    }

    /// 將用戶 ID 轉換為 UUID（用於基礎設施層）
    pub fn user_id_to_uuid(user_id: &UserId) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(user_id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_user_id() {
        let id = IdService::generate_user_id();
        // 驗證生成的 ID 可以轉換回 UUID
        assert!(IdService::user_id_to_uuid(&id).is_ok());
    }

    #[test]
    fn test_user_id_from_uuid_string() {
        let uuid_str = "01234567-89ab-cdef-0123-456789abcdef";
        let user_id = IdService::user_id_from_uuid_string(uuid_str).unwrap();
        assert_eq!(user_id.as_str(), uuid_str);
    }

    #[test]
    fn test_invalid_uuid_string() {
        let invalid_uuid = "invalid-uuid";
        let result = IdService::user_id_from_uuid_string(invalid_uuid);
        assert!(result.is_err());
    }

    #[test]
    fn test_round_trip_conversion() {
        let original_id = IdService::generate_user_id();
        let uuid = IdService::user_id_to_uuid(&original_id).unwrap();
        let converted_id = IdService::user_id_from_uuid_string(&uuid.to_string()).unwrap();
        assert_eq!(original_id.as_str(), converted_id.as_str());
    }
}
