use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime as DateTime;
use crate::entity::{user, invite_code};
#[derive(Serialize)]
pub struct UserDto{
    pub id: i32,
    pub username: String,
    pub role: String,
    pub disabled: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl From<user::Model> for UserDto{
    fn from(user: user::Model) -> Self {
        Self{
            id: user.id,
            username: user.username,
            role: user.role,
            disabled: user.disabled,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }

}

#[derive(Serialize, Deserialize)]
pub struct InviteCodeDto {
    pub id: i32,
    pub code: String,
    pub group_id: i32,
    pub created_by: i32,
    pub max_users: Option<i32>,
    pub used_count: i32,
    pub expires_at: Option<DateTime>,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl From<invite_code::Model> for InviteCodeDto {
    fn from(code: invite_code::Model) -> Self {
        Self {
            id: code.id,
            code: code.code,
            group_id: code.group_id,
            created_by: code.created_by,
            max_users: code.max_users,
            used_count: code.used_count,
            expires_at: code.expires_at,
            is_active: code.is_active,
            created_at: code.created_at,
            updated_at: code.updated_at,
        }
    }
}