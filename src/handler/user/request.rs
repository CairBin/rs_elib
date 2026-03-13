use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRoleRequest{
    pub role: String,
}

#[derive(Deserialize)]
pub struct UpdateUserPasswordRequest{
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateUserDisabledRequest{
    pub disabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateUserProfileRequest {
    pub password: Option<String>,
}

