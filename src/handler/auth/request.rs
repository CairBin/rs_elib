use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest{
    pub username: String,
    pub password: String,
}


#[derive(Deserialize)]
pub struct RegisterRequest{
    pub username: String,
    pub password: String,
}