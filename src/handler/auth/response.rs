use serde::Serialize;

#[derive(Serialize)]
pub struct LoginResponse{
    pub token: String,
    pub user: super::dto::UserDto,
}