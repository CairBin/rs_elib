use sea_orm::prelude::DateTime;
use serde::Serialize;
use crate::entity::user;

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