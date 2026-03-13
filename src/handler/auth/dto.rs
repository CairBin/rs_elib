use serde::Serialize;
use crate::entity::prelude::UserModel;

#[derive(Serialize)]
pub struct UserDto{
    pub id: i32,
    pub username: String,
    pub role: String,
}


impl UserDto{
    pub fn from_entity(entity: UserModel) -> Self{
        Self{
            id: entity.id,
            username: entity.username.clone(),
            role: entity.role.clone(),
        }
    }
}