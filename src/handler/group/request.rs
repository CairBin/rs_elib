use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AddUserToGroupRequest {
    pub user_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct AddBookToGroupRequest {
    pub book_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CreateInviteCodeRequest {
    pub max_users: Option<i32>,
    pub expires_in_days: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct JoinWithInviteCodeRequest {
    pub code: String,
}